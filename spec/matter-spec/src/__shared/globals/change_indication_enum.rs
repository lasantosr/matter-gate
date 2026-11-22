crate::define_matter_enum! {
    /// ChangeIndicationEnum exhaustive specification enum.
    pub enum ChangeIndicationEnum {
        Ok = 0x0,
        Warning = 0x1,
        Critical = 0x2,
    }
    repr = u8;
    attr_variant = U8;
    default = Ok;
}
