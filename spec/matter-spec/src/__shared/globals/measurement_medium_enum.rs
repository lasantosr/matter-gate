crate::define_matter_enum! {
    /// MeasurementMediumEnum exhaustive specification enum.
    pub enum MeasurementMediumEnum {
        Air = 0x0,
        Water = 0x1,
        Soil = 0x2,
    }
    repr = u8;
    attr_variant = U8;
    default = Air;
}
