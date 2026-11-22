crate::define_matter_enum! {
    /// TestGlobalEnum exhaustive specification enum.
    pub enum TestGlobalEnum {
        SomeValue = 0x0,
        SomeOtherValue = 0x1,
        FinalValue = 0x2,
    }
    repr = u8;
    attr_variant = U8;
    default = SomeValue;
}
