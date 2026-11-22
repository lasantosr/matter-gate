crate::define_matter_enum! {
    /// WebRtcEndReasonEnum exhaustive specification enum.
    pub enum WebRtcEndReasonEnum {
        IceFailed = 0x0,
        IceTimeout = 0x1,
        UserHangup = 0x2,
        UserBusy = 0x3,
        Replaced = 0x4,
        NoUserMedia = 0x5,
        InviteTimeout = 0x6,
        AnsweredElsewhere = 0x7,
        OutOfResources = 0x8,
        MediaTimeout = 0x9,
        LowPower = 0xA,
        PrivacyMode = 0xB,
        UnknownReason = 0xC,
    }
    repr = u8;
    attr_variant = U8;
    default = IceFailed;
}
