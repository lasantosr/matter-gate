crate::define_matter_bitmap! {
    /// OperationalStatus bitmap.
    pub struct OperationalStatus(u8);
    attr_variant = U8;
    display_fmt = "{:#04x}";
    flags {
        /// Bitmask flag for `Global` (`0x3`).
        GLOBAL : is_global = 0x3;
        /// Bitmask flag for `Lift` (`0xC`).
        LIFT : is_lift = 0xC;
        /// Bitmask flag for `Tilt` (`0x30`).
        TILT : is_tilt = 0x30;
    }
}
