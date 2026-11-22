crate::define_matter_bitmap! {
    /// LatchControlModesBitmap bitmap.
    pub struct LatchControlModesBitmap(u8);
    attr_variant = U8;
    display_fmt = "{:#04x}";
    flags {
        /// Bitmask flag for `RemoteLatching` (`0x1`).
        REMOTE_LATCHING : is_remote_latching = 0x1;
        /// Bitmask flag for `RemoteUnlatching` (`0x2`).
        REMOTE_UNLATCHING : is_remote_unlatching = 0x2;
    }
}
