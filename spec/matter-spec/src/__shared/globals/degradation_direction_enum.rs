crate::define_matter_enum! {
    /// DegradationDirectionEnum exhaustive specification enum.
    pub enum DegradationDirectionEnum {
        Up = 0x0,
        Down = 0x1,
    }
    repr = u8;
    attr_variant = U8;
    default = Up;
}
