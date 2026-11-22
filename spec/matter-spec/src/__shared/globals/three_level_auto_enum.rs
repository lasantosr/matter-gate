crate::define_matter_enum! {
    /// ThreeLevelAutoEnum exhaustive specification enum.
    pub enum ThreeLevelAutoEnum {
        Auto = 0x0,
        Low = 0x1,
        Medium = 0x2,
        High = 0x3,
    }
    repr = u8;
    attr_variant = U8;
    default = Auto;
}
