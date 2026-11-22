crate::define_matter_enum! {
    /// ProductFinishEnum exhaustive specification enum.
    pub enum ProductFinishEnum {
        Other = 0x0,
        Matte = 0x1,
        Satin = 0x2,
        Polished = 0x3,
        Rugged = 0x4,
        Fabric = 0x5,
    }
    repr = u8;
    attr_variant = U8;
    default = Other;
}
