crate::define_matter_enum! {
    /// ColorEnum exhaustive specification enum.
    pub enum ColorEnum {
        Black = 0x0,
        Navy = 0x1,
        Green = 0x2,
        Teal = 0x3,
        Maroon = 0x4,
        Purple = 0x5,
        Olive = 0x6,
        Gray = 0x7,
        Blue = 0x8,
        Lime = 0x9,
        Aqua = 0xA,
        Red = 0xB,
        Fuchsia = 0xC,
        Yellow = 0xD,
        White = 0xE,
        Nickel = 0xF,
        Chrome = 0x10,
        Brass = 0x11,
        Copper = 0x12,
        Silver = 0x13,
        Gold = 0x14,
    }
    repr = u8;
    attr_variant = U8;
    default = Black;
}
