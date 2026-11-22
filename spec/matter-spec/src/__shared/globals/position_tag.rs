crate::define_matter_enum! {
    /// PositionTag exhaustive specification enum.
    pub enum PositionTag {
        Left = 0x0,
        Right = 0x1,
        Top = 0x2,
        Bottom = 0x3,
        Middle = 0x4,
        Row = 0x5,
        Column = 0x6,
    }
    repr = u8;
    attr_variant = U8;
    default = Left;
}
