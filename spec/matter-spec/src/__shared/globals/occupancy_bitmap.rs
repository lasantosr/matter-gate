crate::define_matter_bitmap! {
    /// OccupancyBitmap bitmap.
    pub struct OccupancyBitmap(u8);
    attr_variant = U8;
    display_fmt = "{:#04x}";
    flags {
        /// Bitmask flag for `Occupied` (`0x1`).
        OCCUPIED : is_occupied = 0x1;
    }
}
