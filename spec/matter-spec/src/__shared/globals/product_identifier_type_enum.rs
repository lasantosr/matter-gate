crate::define_matter_enum! {
    /// ProductIdentifierTypeEnum exhaustive specification enum.
    pub enum ProductIdentifierTypeEnum {
        Upc = 0x0,
        Gtin8 = 0x1,
        Ean = 0x2,
        Gtin14 = 0x3,
        Oem = 0x4,
    }
    repr = u8;
    attr_variant = U8;
    default = Upc;
}
