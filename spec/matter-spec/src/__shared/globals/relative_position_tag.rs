crate::define_matter_enum! {
    /// RelativePositionTag exhaustive specification enum.
    pub enum RelativePositionTag {
        Under = 0x0,
        NextTo = 0x1,
        Around = 0x2,
        On = 0x3,
        Above = 0x4,
        FrontOf = 0x5,
        Behind = 0x6,
    }
    repr = u8;
    attr_variant = U8;
    default = Under;
}
