//! Static schema registry (`SpecRegistry`), builder-configured `RegistryValidator`, and cluster/endpoint/node checks.

use super::{
    IntegerMode, MAX_STACK_MEMBERS, StackBitset, ValidationMode,
    conformance::{check_choice_group, eval_conformance, is_disallowed},
    descriptor_checks::{
        decode_and_check_client_list, decode_and_check_device_types, extract_feature_map, is_global_attribute,
        validate_descriptor_consistency, validate_device_type_reqs, validate_include_item,
        validate_node_parts_integrity,
    },
    descriptors::{ClusterDescriptor, ConformanceResult, DeviceTypeDescriptor},
    value::{validate_global_attribute_value, validate_struct_fields, validate_value_with_ctx},
};
use crate::{
    error::{AttributeError, ClusterError, CommandError, EndpointError, EventError, NodeError},
    model::{Attribute, AttributeId, Cluster, ClusterId, Command, DeviceTypeId, Endpoint, Event, MatterStatus, Node},
};

/// Read-only static schema registry for zero-allocation validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecRegistry {
    /// Whether `EventList` (`0xFFFA`) is defined in this release's global attributes and expected in `FullSnapshot`
    /// mode.
    pub(crate) expects_event_list: bool,
    /// Sorted slice of cluster descriptors (ascending by `ClusterId`).
    pub(crate) clusters: &'static [ClusterDescriptor],
    /// Sorted slice of device type descriptors (ascending by `DeviceTypeId`).
    pub(crate) device_types: &'static [DeviceTypeDescriptor],
}

impl SpecRegistry {
    /// Returns all cluster descriptors in this registry, sorted by [`ClusterId`].
    #[must_use]
    pub const fn clusters(&self) -> &'static [ClusterDescriptor] {
        self.clusters
    }

    /// Returns all device type descriptors in this registry, sorted by [`DeviceTypeId`].
    #[must_use]
    pub const fn device_types(&self) -> &'static [DeviceTypeDescriptor] {
        self.device_types
    }

    /// Returns a [`RegistryValidator`] configured for [`ValidationMode::FullSnapshot`] (`enforce_provisional: false`).
    #[must_use]
    pub const fn validator(&self) -> RegistryValidator<'_> {
        RegistryValidator::full(self)
    }

    /// Returns a [`RegistryValidator`] configured for [`ValidationMode::Partial`].
    #[must_use]
    pub const fn partial_validator(&self) -> RegistryValidator<'_> {
        RegistryValidator::partial(self)
    }

    /// Looks up a cluster descriptor by [`ClusterId`] in `O(log N)` time.
    #[must_use]
    pub fn find_cluster(&self, id: ClusterId) -> Option<&'static ClusterDescriptor> {
        let idx = self.clusters.binary_search_by_key(&id.raw(), |c| c.id.raw()).ok()?;
        self.clusters.get(idx)
    }

    /// Looks up a device type descriptor by [`DeviceTypeId`] in `O(log N)` time.
    #[must_use]
    pub fn find_device_type(&self, id: DeviceTypeId) -> Option<&'static DeviceTypeDescriptor> {
        let idx = self.device_types.binary_search_by_key(&id.raw(), |d| d.id.raw()).ok()?;
        self.device_types.get(idx)
    }

    #[cfg(test)]
    pub(super) fn validate_cluster_ref_with_mode(
        &self,
        cluster: &Cluster,
        mode: ValidationMode,
    ) -> Result<(), ClusterError> {
        self.validate_cluster_ref_with_modes(cluster, mode, IntegerMode::Widening)
    }

    fn validate_attribute_ref_with_int_mode(
        &self,
        cluster_id: ClusterId,
        attr: &Attribute,
        int_mode: IntegerMode,
    ) -> Result<(), AttributeError> {
        let cluster = self
            .find_cluster(cluster_id)
            .ok_or(AttributeError::UnsupportedCluster(cluster_id))?;
        if let Some(desc) = cluster.attributes.iter().find(|a| a.id == attr.id) {
            validate_value_with_ctx(&attr.value, &desc.constraint, 0, None, int_mode).map_err(|(field_id, status)| {
                AttributeError::InvalidValue {
                    id: attr.id,
                    field_id,
                    status,
                }
            })
        } else if is_global_attribute(attr.id) {
            validate_global_attribute_value(attr.id, &attr.value, int_mode).map_err(|status| {
                AttributeError::InvalidValue {
                    id: attr.id,
                    field_id: None,
                    status,
                }
            })
        } else {
            Err(AttributeError::UnsupportedAttribute(attr.id))
        }
    }

    fn validate_command_ref_with_int_mode(&self, cmd: &Command, int_mode: IntegerMode) -> Result<(), CommandError> {
        let cluster = self
            .find_cluster(cmd.cluster_id)
            .ok_or(CommandError::UnsupportedCluster(cmd.cluster_id))?;
        let Some(desc) = cluster.commands.iter().find(|c| c.id == cmd.command_id) else {
            return Err(CommandError::UnsupportedCommand(cmd.command_id));
        };
        if cmd.direction != desc.direction {
            return Err(CommandError::DirectionMismatch {
                expected: desc.direction,
                actual: cmd.direction,
            });
        }
        validate_struct_fields(&cmd.payload, desc.fields, true, 0, None, int_mode).map_err(|(field_id, status)| {
            CommandError::InvalidPayload {
                id: cmd.command_id,
                field_id,
                status,
            }
        })
    }

    fn validate_event_ref_with_int_mode(&self, event: &Event, int_mode: IntegerMode) -> Result<(), EventError> {
        let cluster = self
            .find_cluster(event.cluster_id)
            .ok_or(EventError::UnsupportedCluster(event.cluster_id))?;
        let Some(desc) = cluster.events.iter().find(|e| e.id == event.event_id) else {
            return Err(EventError::UnsupportedEvent(event.event_id));
        };
        if event.priority != desc.priority {
            return Err(EventError::PriorityMismatch {
                expected: desc.priority,
                actual: event.priority,
            });
        }
        validate_struct_fields(&event.payload, desc.fields, false, 0, None, int_mode).map_err(|(field_id, status)| {
            EventError::InvalidPayload {
                id: event.event_id,
                field_id,
                status,
            }
        })
    }

    fn validate_cluster_ref_with_modes(
        &self,
        cluster: &Cluster,
        mode: ValidationMode,
        int_mode: IntegerMode,
    ) -> Result<(), ClusterError> {
        let desc = self
            .find_cluster(cluster.id)
            .ok_or(ClusterError::UnsupportedCluster(cluster.id))?;

        let feature_map = extract_feature_map(cluster, int_mode);
        let mut seen = StackBitset::new();
        let attrs = &cluster.attributes;

        // 1. Check duplicates and validate each present attribute.
        for (idx, attr) in attrs.iter().enumerate() {
            if let Some((desc_idx, attr_desc)) = desc.attributes.iter().enumerate().find(|(_, a)| a.id == attr.id) {
                check_attr_duplicate(&mut seen, attrs, idx, desc_idx, attr.id)?;

                let conf = eval_conformance(attr_desc.conformance, feature_map, Some(cluster));
                if is_disallowed(conf, mode) {
                    return Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(
                        attr.id,
                    )));
                }
                validate_value_with_ctx(&attr.value, &attr_desc.constraint, feature_map, Some(cluster), int_mode)
                    .map_err(|(field_id, status)| {
                        ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                            id: attr.id,
                            field_id,
                            status,
                        })
                    })?;
            } else if is_global_attribute(attr.id) {
                if attrs
                    .get(..idx)
                    .is_some_and(|prevs| prevs.iter().any(|prev| prev.id == attr.id))
                {
                    return Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                        id: attr.id,
                        field_id: None,
                        status: MatterStatus::ConstraintError,
                    }));
                }
                validate_global_attribute_value(attr.id, &attr.value, int_mode).map_err(|status| {
                    ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                        id: attr.id,
                        field_id: None,
                        status,
                    })
                })?;
            } else {
                return Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(
                    attr.id,
                )));
            }
        }

        // 2. FullSnapshot checks: mandatory attributes, EventList (v1.1..=v1.3), and ChoiceGroups.
        if matches!(mode, ValidationMode::FullSnapshot { .. }) {
            for attr_desc in desc.attributes {
                let conf = eval_conformance(attr_desc.conformance, feature_map, Some(cluster));
                if conf == ConformanceResult::Mandatory && !attrs.iter().any(|a| a.id == attr_desc.id) {
                    return Err(ClusterError::MissingMandatoryAttribute(attr_desc.id));
                }
            }

            if self.expects_event_list
                && !desc.events.is_empty()
                && !attrs.iter().any(|a| a.id == AttributeId::EVENT_LIST)
            {
                return Err(ClusterError::MissingMandatoryAttribute(AttributeId::EVENT_LIST));
            }

            for group in desc.choice_groups {
                if !check_choice_group(group, |id| attrs.iter().any(|a| a.id.raw() == id)) {
                    let first_id = group.members.first().copied().unwrap_or(0);
                    return Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                        id: AttributeId::new(first_id),
                        field_id: None,
                        status: MatterStatus::ConstraintError,
                    }));
                }
            }
        }

        Ok(())
    }

    fn validate_endpoint_ref_with_modes(
        &self,
        endpoint: &Endpoint,
        mode: ValidationMode,
        int_mode: IntegerMode,
    ) -> Result<(), EndpointError> {
        if !endpoint.id.is_valid() {
            return Err(EndpointError::InvalidEndpointId(endpoint.id));
        }

        let Some(descriptor) = endpoint.descriptor() else {
            return Err(EndpointError::MissingMandatoryServerCluster(ClusterId::DESCRIPTOR));
        };

        // Decode and deduplicate DeviceTypeList (0x0000) in a single zero-alloc pass.
        let device_types = decode_and_check_device_types(descriptor, int_mode)?;

        // Duplicate server cluster IDs.
        for (i, c) in endpoint.clusters.iter().enumerate() {
            if endpoint
                .clusters
                .get(..i)
                .is_some_and(|prevs| prevs.iter().any(|prev| prev.id == c.id))
            {
                return Err(EndpointError::DuplicateCluster(c.id));
            }
        }

        // Decode and deduplicate Descriptor::ClientList (0x0002) once, if present.
        let client_list = decode_and_check_client_list(descriptor, int_mode)?;

        // Validate each device type's cluster requirements, choice groups, and includes.
        device_types.try_for_each(|dt_id| {
            let dt_desc = self
                .find_device_type(dt_id)
                .ok_or(EndpointError::UnsupportedDeviceType(dt_id))?;
            validate_device_type_reqs(endpoint, &client_list, dt_desc, mode)?;
            for inc in dt_desc.includes {
                validate_include_item(endpoint, inc, mode, int_mode)?;
            }
            Ok(())
        })?;

        // Cross-check Descriptor ServerList (0x0001), PartsList (0x0003), and TagList (0x0004) if present.
        validate_descriptor_consistency(endpoint, descriptor, int_mode)?;

        // Validate each registered server cluster; extra/MEI clusters not in registry are permitted.
        for cluster in &endpoint.clusters {
            if self.find_cluster(cluster.id).is_some() {
                self.validate_cluster_ref_with_modes(cluster, mode, int_mode)
                    .map_err(|source| EndpointError::InvalidCluster { id: cluster.id, source })?;
            }
        }

        Ok(())
    }

    fn validate_node_ref_with_modes(
        &self,
        node: &Node,
        mode: ValidationMode,
        int_mode: IntegerMode,
    ) -> Result<(), NodeError> {
        if node.root_endpoint().is_none() {
            return Err(NodeError::MissingRootEndpoint);
        }
        for (i, ep_a) in node.endpoints.iter().enumerate() {
            for ep_b in node.endpoints.iter().skip(i.saturating_add(1)) {
                if ep_a.id == ep_b.id {
                    return Err(NodeError::DuplicateEndpoint(ep_a.id));
                }
            }
        }
        for ep in &node.endpoints {
            self.validate_endpoint_ref_with_modes(ep, mode, int_mode)
                .map_err(|source| NodeError::InvalidEndpoint { id: ep.id, source })?;
        }
        if matches!(mode, ValidationMode::FullSnapshot { .. }) {
            validate_node_parts_integrity(node, int_mode)?;
        }
        Ok(())
    }
}

/// Zero-cost, builder-configured validator bound to a [`SpecRegistry`].
///
/// Defaults to [`ValidationMode::FullSnapshot`] (`enforce_provisional: false`) and [`IntegerMode::Widening`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryValidator<'a> {
    registry: &'a SpecRegistry,
    mode: ValidationMode,
    integer_mode: IntegerMode,
}

impl<'a> RegistryValidator<'a> {
    /// Creates a [`RegistryValidator`] in [`ValidationMode::FullSnapshot`] (`enforce_provisional: false`).
    #[must_use]
    pub const fn new(registry: &'a SpecRegistry) -> Self {
        Self::full(registry)
    }

    /// Creates a [`RegistryValidator`] in [`ValidationMode::FullSnapshot`] (`enforce_provisional: false`).
    #[must_use]
    pub const fn full(registry: &'a SpecRegistry) -> Self {
        Self {
            registry,
            mode: ValidationMode::FullSnapshot {
                enforce_provisional: false,
            },
            integer_mode: IntegerMode::Widening,
        }
    }

    /// Creates a [`RegistryValidator`] in [`ValidationMode::Partial`].
    #[must_use]
    pub const fn partial(registry: &'a SpecRegistry) -> Self {
        Self {
            registry,
            mode: ValidationMode::Partial,
            integer_mode: IntegerMode::Widening,
        }
    }

    /// Creates a [`RegistryValidator`] with an explicit [`ValidationMode`].
    #[must_use]
    pub const fn with_mode(self, mode: ValidationMode) -> Self {
        Self {
            registry: self.registry,
            mode,
            integer_mode: self.integer_mode,
        }
    }

    /// Sets the [`IntegerMode`] used when validating integer wire tags.
    #[must_use]
    pub const fn with_integer_mode(self, integer_mode: IntegerMode) -> Self {
        Self {
            registry: self.registry,
            mode: self.mode,
            integer_mode,
        }
    }

    /// Switches the validator to [`ValidationMode::Partial`].
    #[must_use]
    pub const fn partial_mode(self) -> Self {
        self.with_mode(ValidationMode::Partial)
    }

    /// Switches the validator to [`ValidationMode::FullSnapshot`] (`enforce_provisional: false`).
    #[must_use]
    pub const fn full_mode(self) -> Self {
        self.with_mode(ValidationMode::FullSnapshot {
            enforce_provisional: false,
        })
    }

    /// Sets `enforce_provisional` when in [`ValidationMode::FullSnapshot`].
    ///
    /// Has no effect when in [`ValidationMode::Partial`].
    #[must_use]
    pub const fn enforce_provisional(self, enforce: bool) -> Self {
        match self.mode {
            ValidationMode::FullSnapshot { .. } => self.with_mode(ValidationMode::FullSnapshot {
                enforce_provisional: enforce,
            }),
            ValidationMode::Partial => self,
        }
    }

    /// Returns the active [`ValidationMode`].
    #[must_use]
    pub const fn mode(&self) -> ValidationMode {
        self.mode
    }

    /// Returns the active [`IntegerMode`].
    #[must_use]
    pub const fn integer_mode(&self) -> IntegerMode {
        self.integer_mode
    }

    /// Validates a borrowed [`Attribute`] on `cluster_id` without allocating or mutating.
    pub fn attribute(&self, cluster_id: ClusterId, attr: &Attribute) -> Result<(), AttributeError> {
        self.registry
            .validate_attribute_ref_with_int_mode(cluster_id, attr, self.integer_mode)
    }

    /// Validates a borrowed [`Command`] without allocating or mutating.
    pub fn command(&self, cmd: &Command) -> Result<(), CommandError> {
        self.registry.validate_command_ref_with_int_mode(cmd, self.integer_mode)
    }

    /// Validates a borrowed [`Event`] without allocating or mutating.
    pub fn event(&self, event: &Event) -> Result<(), EventError> {
        self.registry.validate_event_ref_with_int_mode(event, self.integer_mode)
    }

    /// Validates a borrowed [`Cluster`] using the configured [`ValidationMode`] and [`IntegerMode`].
    pub fn cluster(&self, cluster: &Cluster) -> Result<(), ClusterError> {
        self.registry
            .validate_cluster_ref_with_modes(cluster, self.mode, self.integer_mode)
    }

    /// Validates a borrowed [`Endpoint`] using the configured [`ValidationMode`] and [`IntegerMode`].
    pub fn endpoint(&self, endpoint: &Endpoint) -> Result<(), EndpointError> {
        self.registry
            .validate_endpoint_ref_with_modes(endpoint, self.mode, self.integer_mode)
    }

    /// Validates a borrowed [`Node`] using the configured [`ValidationMode`] and [`IntegerMode`].
    pub fn node(&self, node: &Node) -> Result<(), NodeError> {
        self.registry
            .validate_node_ref_with_modes(node, self.mode, self.integer_mode)
    }
}

/// Rejects duplicate cluster attributes using `seen` (with linear fallback above `MAX_STACK_MEMBERS`).
fn check_attr_duplicate(
    seen: &mut StackBitset,
    attrs: &[Attribute],
    idx: usize,
    desc_idx: usize,
    id: AttributeId,
) -> Result<(), ClusterError> {
    let is_dup = if desc_idx < MAX_STACK_MEMBERS {
        !seen.insert(desc_idx)
    } else {
        attrs
            .get(..idx)
            .is_some_and(|prevs| prevs.iter().any(|prev| prev.id == id))
    };
    if is_dup {
        return Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
            id,
            field_id: None,
            status: MatterStatus::ConstraintError,
        }));
    }
    Ok(())
}
