crate::define_matter_enum! {
    /// TariffUnitEnum exhaustive specification enum.
    pub enum TariffUnitEnum {
        KWh = 0x0,
        KVAh = 0x1,
    }
    repr = u8;
    attr_variant = U8;
    default = KWh;
}
