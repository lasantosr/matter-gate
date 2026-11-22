crate::define_matter_enum! {
    /// TariffPriceTypeEnum exhaustive specification enum.
    pub enum TariffPriceTypeEnum {
        Standard = 0x0,
        Critical = 0x1,
        Virtual = 0x2,
        Incentive = 0x3,
        IncentiveSignal = 0x4,
    }
    repr = u8;
    attr_variant = U8;
    default = Standard;
}
