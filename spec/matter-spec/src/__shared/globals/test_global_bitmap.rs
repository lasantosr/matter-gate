crate::define_matter_bitmap! {
    /// TestGlobalBitmap bitmap.
    pub struct TestGlobalBitmap(u32);
    attr_variant = U32;
    display_fmt = "{:#010x}";
    flags {
        /// Bitmask flag for `FirstBit` (`0x1`).
        FIRST_BIT : is_first_bit = 0x1;
        /// Bitmask flag for `SecondBit` (`0x2`).
        SECOND_BIT : is_second_bit = 0x2;
    }
}
