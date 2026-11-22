crate::define_matter_enum! {
    /// MeasurementTypeEnum exhaustive specification enum.
    pub enum MeasurementTypeEnum {
        Unspecified = 0x0,
        Voltage = 0x1,
        ActiveCurrent = 0x2,
        ReactiveCurrent = 0x3,
        ApparentCurrent = 0x4,
        ActivePower = 0x5,
        ReactivePower = 0x6,
        ApparentPower = 0x7,
        RmsVoltage = 0x8,
        RmsCurrent = 0x9,
        RmsPower = 0xA,
        Frequency = 0xB,
        PowerFactor = 0xC,
        NeutralCurrent = 0xD,
        ElectricalEnergy = 0xE,
        ReactiveEnergy = 0xF,
        ApparentEnergy = 0x10,
        SoilMoisture = 0x11,
    }
    repr = u16;
    attr_variant = U16;
    default = Unspecified;
}
