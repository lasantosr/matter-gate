crate::define_matter_enum! {
    /// LevelValueEnum exhaustive specification enum.
    pub enum LevelValueEnum {
        Unknown = 0x0,
        Low = 0x1,
        Medium = 0x2,
        High = 0x3,
        Critical = 0x4,
    }
    repr = u8;
    attr_variant = U8;
    default = Unknown;
}
