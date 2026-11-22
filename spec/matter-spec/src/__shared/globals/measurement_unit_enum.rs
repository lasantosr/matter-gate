crate::define_matter_enum! {
    /// MeasurementUnitEnum exhaustive specification enum.
    pub enum MeasurementUnitEnum {
        Ppm = 0x0,
        Ppb = 0x1,
        Ppt = 0x2,
        Mgm3 = 0x3,
        Ugm3 = 0x4,
        Ngm3 = 0x5,
        Pm3 = 0x6,
        Bqm3 = 0x7,
    }
    repr = u8;
    attr_variant = U8;
    default = Ppm;
}
