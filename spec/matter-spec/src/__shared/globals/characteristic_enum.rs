crate::define_matter_enum! {
    /// CharacteristicEnum exhaustive specification enum.
    pub enum CharacteristicEnum {
        ForcedSubtitles = 0x0,
        DescribesVideo = 0x1,
        EasyToRead = 0x2,
        FrameBased = 0x3,
        MainProgram = 0x4,
        OriginalContent = 0x5,
        VoiceOverTranslation = 0x6,
        Caption = 0x7,
        Subtitle = 0x8,
        Alternate = 0x9,
        Supplementary = 0xA,
        Commentary = 0xB,
        DubbedTranslation = 0xC,
        Description = 0xD,
        Metadata = 0xE,
        EnhancedAudioIntelligibility = 0xF,
        Emergency = 0x10,
        Karaoke = 0x11,
    }
    repr = u8;
    attr_variant = U8;
    default = ForcedSubtitles;
}
