//! Declarative macros for `MatterDescribe` and self-describing variant dispatch selection.

macro_rules! impl_matter_describe {
    (cluster, $ty:ident, $is_empty:expr) => {
        $crate::impl_matter_describe!(
            @concrete $ty,
            $crate::dispatch::EntityDescriptor::Cluster {
                cluster: <$ty>::NAME,
                cluster_id: <$ty>::CLUSTER_ID.raw(),
            },
            $is_empty
        );
    };
    (device, $ty:ident, $is_empty:expr) => {
        $crate::impl_matter_describe!(
            @concrete $ty,
            $crate::dispatch::EntityDescriptor::Device {
                device_type: <$ty>::NAME,
                device_type_id: <$ty>::DEVICE_TYPE_ID.raw(),
            },
            $is_empty
        );
    };
    (@concrete $ty:ident, $desc:expr, $is_empty:expr) => {
        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::MatterDescribe for $ty {
            const STATIC_SCHEMA: $crate::dispatch::StaticEntitySchema =
                $crate::dispatch::StaticEntitySchema::concrete($desc, $is_empty);

            #[inline]
            fn descriptor(&self) -> $crate::dispatch::EntityDescriptor {
                $desc
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                $is_empty
            }

            #[cfg(feature = "serde")]
            fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::dispatch::deserialize_concrete_described(deserializer, $desc, $is_empty)
            }
        }
    };
}

macro_rules! impl_select_described {
    (attr, $kind:literal) => {
        $crate::impl_select_described!(@tuple3 $kind);
    };
    (ev, $kind:literal) => {
        $crate::impl_select_described!(@tuple3 $kind);
    };
    (cmd, $kind:literal) => {
        #[cfg(all(feature = "dispatch", feature = "serde"))]
        pub(crate) fn select_described(
            item_name: Option<&str>,
            item_id: Option<u32>,
            payload: $crate::dispatch::BufferedPayload,
        ) -> Result<Self, alloc::string::String> {
            let deserialize = match (item_name, item_id) {
                (Some(name), maybe_id) => {
                    let (_, id, _, f) = Self::DESCRIBED_VARIANTS
                        .iter()
                        .copied()
                        .find(|&(n, _, _, _)| n == name)
                        .ok_or_else(|| alloc::format!("unknown {} '{name}'", $kind))?;
                    if let Some(expected_id) = maybe_id {
                        if id != expected_id {
                            return Err(alloc::format!(
                                "conflicting {} '{name}' and {}_id {expected_id}",
                                $kind,
                                $kind
                            ));
                        }
                    }
                    f
                }
                (None, Some(id)) => Self::DESCRIBED_VARIANTS
                    .iter()
                    .copied()
                    .find(|&(_, vid, _, _)| vid == id)
                    .map(|(_, _, _, f)| f)
                    .ok_or_else(|| alloc::format!("unknown {}_id {id}", $kind))?,
                (None, None) => return Err(alloc::format!("missing {} discriminator", $kind)),
            };

            deserialize(payload)
        }
    };
    (@tuple3 $kind:literal) => {
        #[cfg(all(feature = "dispatch", feature = "serde"))]
        pub(crate) fn select_described(
            item_name: Option<&str>,
            item_id: Option<u32>,
            payload: $crate::dispatch::BufferedPayload,
        ) -> Result<Self, alloc::string::String> {
            let deserialize = match (item_name, item_id) {
                (Some(name), maybe_id) => {
                    let (_, id, f) = Self::DESCRIBED_VARIANTS
                        .iter()
                        .copied()
                        .find(|&(n, _, _)| n == name)
                        .ok_or_else(|| alloc::format!("unknown {} '{name}'", $kind))?;
                    if let Some(expected_id) = maybe_id {
                        if id != expected_id {
                            return Err(alloc::format!(
                                "conflicting {} '{name}' and {}_id {expected_id}",
                                $kind,
                                $kind
                            ));
                        }
                    }
                    f
                }
                (None, Some(id)) => Self::DESCRIBED_VARIANTS
                    .iter()
                    .copied()
                    .find(|&(_, vid, _)| vid == id)
                    .map(|(_, _, f)| f)
                    .ok_or_else(|| alloc::format!("unknown {}_id {id}", $kind))?,
                (None, None) => return Err(alloc::format!("missing {} discriminator", $kind)),
            };

            deserialize(payload)
        }
    };
}

/// Implements `*Meta` and [`MatterDescribe`](crate::dispatch::MatterDescribe) boilerplate for cluster member enums.
macro_rules! impl_cluster_enum_dispatch {
    (attribute, $enum_ty:ident, $cluster:ident, [$($variant:ident),* $(,)?]) => {
        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::AttributeMeta for $enum_ty {
            #[inline]
            fn cluster_id(&self) -> $crate::model::ClusterId {
                Self::CLUSTER_ID
            }

            #[inline]
            fn cluster_name(&self) -> &'static str {
                $cluster::NAME
            }

            #[inline]
            fn attribute_id(&self) -> $crate::model::AttributeId {
                self.id()
            }

            #[inline]
            fn attribute_name(&self) -> &'static str {
                match self {
                    $(Self::$variant(_) => attributes::$variant::NAME,)*
                }
            }
        }

        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::MatterDescribe for $enum_ty {
            const STATIC_SCHEMA: $crate::dispatch::StaticEntitySchema =
                $crate::dispatch::StaticEntitySchema::cluster_enum(
                    $crate::dispatch::EntityKind::Attribute,
                    $cluster::NAME,
                    $cluster::CLUSTER_ID.raw(),
                );

            #[inline]
            fn descriptor(&self) -> $crate::dispatch::EntityDescriptor {
                $crate::dispatch::EntityDescriptor::Attribute {
                    cluster: $cluster::NAME,
                    cluster_id: $cluster::CLUSTER_ID.raw(),
                    attribute: <Self as $crate::dispatch::AttributeMeta>::attribute_name(self),
                    attribute_id: self.id().raw(),
                }
            }

            #[cfg(feature = "serde")]
            fn serialize_payload<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    $(Self::$variant(inner) => serde::Serialize::serialize(inner, serializer),)*
                }
            }

            #[cfg(feature = "serde")]
            fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::dispatch::deserialize_enum_described(
                    deserializer,
                    $crate::dispatch::EntityKind::Attribute,
                    Some(($cluster::NAME, $cluster::CLUSTER_ID.raw())),
                    |_, _, item_name, item_id, payload| Self::select_described(item_name, item_id, payload),
                )
            }
        }
    };
    (
        command,
        $enum_ty:ident,
        $cluster:ident,
        empty = [$($empty:ident),* $(,)?],
        payload = [$($payload:ident),* $(,)?] $(,)?
    ) => {
        $crate::impl_cluster_enum_dispatch!(
            @command_common
            $enum_ty,
            $cluster,
            empty = [$($empty),*],
            payload = [$($payload),*],
            extra = {
                #[cfg(feature = "serde")]
                fn deserialize_invocation<'de, D>(
                    deserializer: D,
                ) -> Result<($crate::model::EndpointId, Self), D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    $crate::dispatch::deserialize_enum_invocation(
                        deserializer,
                        $crate::dispatch::EntityKind::Command,
                        Some(($cluster::NAME, $cluster::CLUSTER_ID.raw())),
                        |_, _, item_name, item_id, payload| Self::select_described(item_name, item_id, payload),
                    )
                }
            }
        );
    };
    (
        command_response,
        $enum_ty:ident,
        $cluster:ident,
        empty = [$($empty:ident),* $(,)?],
        payload = [$($payload:ident),* $(,)?] $(,)?
    ) => {
        $crate::impl_cluster_enum_dispatch!(
            @command_common
            $enum_ty,
            $cluster,
            empty = [$($empty),*],
            payload = [$($payload),*],
            extra = {}
        );
    };
    (
        @command_common
        $enum_ty:ident,
        $cluster:ident,
        empty = [$($empty:ident),*],
        payload = [$($payload:ident),*],
        extra = { $($extra:tt)* }
    ) => {
        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::CommandMeta for $enum_ty {
            #[inline]
            fn cluster_id(&self) -> $crate::model::ClusterId {
                Self::CLUSTER_ID
            }

            #[inline]
            fn cluster_name(&self) -> &'static str {
                $cluster::NAME
            }

            #[inline]
            fn command_id(&self) -> $crate::model::CommandId {
                self.id()
            }

            #[inline]
            fn command_name(&self) -> &'static str {
                match self {
                    $(Self::$empty => commands::$empty::NAME,)*
                    $(Self::$payload(_) => commands::$payload::NAME,)*
                }
            }

            #[inline]
            fn direction(&self) -> $crate::model::CommandDirection {
                self.direction()
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                self.is_empty_payload()
            }
        }

        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::MatterDescribe for $enum_ty {
            const STATIC_SCHEMA: $crate::dispatch::StaticEntitySchema =
                $crate::dispatch::StaticEntitySchema::cluster_enum(
                    $crate::dispatch::EntityKind::Command,
                    $cluster::NAME,
                    $cluster::CLUSTER_ID.raw(),
                );

            #[inline]
            fn descriptor(&self) -> $crate::dispatch::EntityDescriptor {
                $crate::dispatch::EntityDescriptor::Command {
                    cluster: $cluster::NAME,
                    cluster_id: $cluster::CLUSTER_ID.raw(),
                    command: <Self as $crate::dispatch::CommandMeta>::command_name(self),
                    command_id: self.id().raw(),
                }
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                Self::is_empty_payload(self)
            }

            #[cfg(feature = "serde")]
            fn serialize_payload<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $crate::impl_cluster_enum_dispatch!(
                    @serialize_payload
                    empty = [$($empty),*],
                    payload = [$($payload),*],
                    self,
                    serializer
                )
            }

            #[cfg(feature = "serde")]
            fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::dispatch::deserialize_enum_described(
                    deserializer,
                    $crate::dispatch::EntityKind::Command,
                    Some(($cluster::NAME, $cluster::CLUSTER_ID.raw())),
                    |_, _, item_name, item_id, payload| Self::select_described(item_name, item_id, payload),
                )
            }

            $($extra)*
        }
    };
    (
        event,
        $enum_ty:ident,
        $cluster:ident,
        empty = [$($empty:ident),* $(,)?],
        payload = [$($payload:ident),* $(,)?] $(,)?
    ) => {
        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::EventMeta for $enum_ty {
            #[inline]
            fn cluster_id(&self) -> $crate::model::ClusterId {
                Self::CLUSTER_ID
            }

            #[inline]
            fn cluster_name(&self) -> &'static str {
                $cluster::NAME
            }

            #[inline]
            fn event_id(&self) -> $crate::model::EventId {
                self.id()
            }

            #[inline]
            fn event_name(&self) -> &'static str {
                match self {
                    $(Self::$empty => events::$empty::NAME,)*
                    $(Self::$payload(_) => events::$payload::NAME,)*
                }
            }

            #[inline]
            fn priority(&self) -> $crate::model::EventPriority {
                self.priority()
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                self.is_empty_payload()
            }
        }

        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::MatterDescribe for $enum_ty {
            const STATIC_SCHEMA: $crate::dispatch::StaticEntitySchema =
                $crate::dispatch::StaticEntitySchema::cluster_enum(
                    $crate::dispatch::EntityKind::Event,
                    $cluster::NAME,
                    $cluster::CLUSTER_ID.raw(),
                );

            #[inline]
            fn descriptor(&self) -> $crate::dispatch::EntityDescriptor {
                $crate::dispatch::EntityDescriptor::Event {
                    cluster: $cluster::NAME,
                    cluster_id: $cluster::CLUSTER_ID.raw(),
                    event: <Self as $crate::dispatch::EventMeta>::event_name(self),
                    event_id: self.id().raw(),
                }
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                Self::is_empty_payload(self)
            }

            #[cfg(feature = "serde")]
            fn serialize_payload<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $crate::impl_cluster_enum_dispatch!(
                    @serialize_payload
                    empty = [$($empty),*],
                    payload = [$($payload),*],
                    self,
                    serializer
                )
            }

            #[cfg(feature = "serde")]
            fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::dispatch::deserialize_enum_described(
                    deserializer,
                    $crate::dispatch::EntityKind::Event,
                    Some(($cluster::NAME, $cluster::CLUSTER_ID.raw())),
                    |_, _, item_name, item_id, payload| Self::select_described(item_name, item_id, payload),
                )
            }
        }
    };
    (@serialize_payload empty = [], payload = [$($payload:ident),*], $self_:expr, $ser:expr) => {
        match $self_ {
            $(Self::$payload(inner) => serde::Serialize::serialize(inner, $ser),)*
        }
    };
    (
        @serialize_payload
        empty = [$first:ident $(, $rest:ident)*],
        payload = [$($payload:ident),*],
        $self_:expr,
        $ser:expr
    ) => {
        match $self_ {
            Self::$first $(| Self::$rest)* => serde::Serialize::serialize(&(), $ser),
            $(Self::$payload(inner) => serde::Serialize::serialize(inner, $ser),)*
        }
    };
}

pub(crate) use impl_cluster_enum_dispatch;
pub(crate) use impl_matter_describe;
pub(crate) use impl_select_described;
