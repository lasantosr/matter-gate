crate::define_matter_enum! {
    /// StreamUsageEnum exhaustive specification enum.
    pub enum StreamUsageEnum {
        Internal = 0x0,
        Recording = 0x1,
        Analysis = 0x2,
        LiveView = 0x3,
    }
    repr = u8;
    attr_variant = U8;
    default = Internal;
}
