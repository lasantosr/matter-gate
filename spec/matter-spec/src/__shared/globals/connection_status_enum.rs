crate::define_matter_enum! {
    /// ConnectionStatusEnum exhaustive specification enum.
    pub enum ConnectionStatusEnum {
        Connected = 0x0,
        NotConnected = 0x1,
    }
    repr = u8;
    attr_variant = U8;
    default = Connected;
}
