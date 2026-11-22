crate::define_matter_enum! {
    /// PowerThresholdSourceEnum exhaustive specification enum.
    pub enum PowerThresholdSourceEnum {
        Contract = 0x0,
        Regulator = 0x1,
        Equipment = 0x2,
    }
    repr = u8;
    attr_variant = U8;
    default = Contract;
}
