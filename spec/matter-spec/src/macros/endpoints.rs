//! Declarative impl macros for generated [`DeviceTypeSpec`](crate::traits::DeviceTypeSpec) boilerplate.

/// Implements [`DeviceTypeSpec`](crate::traits::DeviceTypeSpec) and [`MatterDescribe`](crate::dispatch::MatterDescribe) for a generated device type.
macro_rules! impl_device_type_spec {
    ($name:ident, $view:ident, $is_empty:expr) => {
        impl $crate::traits::DeviceTypeSpec for $name {
            type View<'a> = $view<'a>;

            const DEVICE_TYPE_ID: $crate::model::DeviceTypeId = Self::DEVICE_TYPE_ID;
            const IS_EMPTY: bool = $is_empty;
            const NAME: &'static str = Self::NAME;
            const REVISION: u16 = Self::REVISION;

            #[inline]
            fn into_endpoint(self, id: $crate::model::EndpointId) -> $crate::model::Endpoint {
                Self::into_endpoint(self, id)
            }

            fn view(endpoint: &$crate::model::Endpoint) -> Result<Self::View<'_>, $crate::error::EndpointError> {
                $crate::traits::validate_device_view(
                    endpoint,
                    Self::DEVICE_TYPE_ID,
                    Self::DISALLOWED_CLUSTERS,
                    Self::MANDATORY_SERVER_CLUSTERS,
                    Self::MANDATORY_CLIENT_CLUSTERS,
                )?;
                Ok($view { endpoint })
            }
        }

        $crate::impl_matter_describe!(device, $name, $is_empty);
    };
}

pub(crate) use impl_device_type_spec;
