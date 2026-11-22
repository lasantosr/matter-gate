crate::define_matter_enum! {
    /// AtomicRequestTypeEnum exhaustive specification enum.
    pub enum AtomicRequestTypeEnum {
        BeginWrite = 0x0,
        CommitWrite = 0x1,
        RollbackWrite = 0x2,
    }
    repr = u8;
    attr_variant = U8;
    default = BeginWrite;
}
