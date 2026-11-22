crate::define_matter_enum! {
    /// LocationTag exhaustive specification enum.
    pub enum LocationTag {
        Indoor = 0x0,
        Outdoor = 0x1,
        Inside = 0x2,
        Outside = 0x3,
    }
    repr = u8;
    attr_variant = U8;
    default = Indoor;
}
