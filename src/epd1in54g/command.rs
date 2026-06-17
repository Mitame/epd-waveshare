//! SPI Commands for the Waveshare 1.54" G white/black/red/yellow E-Ink Display
use crate::traits;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    PanelSetting = 0x00,

    PowerSetting = 0x01,
    PowerOff = 0x02,
    PowerOffSequenceSetting = 0x03,
    PowerOn = 0x04,
    BoosterSoftStart = 0x06,
    DeepSleep = 0x07,
    DataStartTransmission1 = 0x10,
    DataStop = 0x11,
    DisplayRefresh = 0x12,
    AutoSequence = 0x17,

    PllControl = 0x30,
    TemperatureSensor = 0x40,
    TemperatureSensorSelection = 0x41,
    TemperatureSensorWrite = 0x42,
    TemperatureSensorRead = 0x43,
    UnknownInit1 = 0x4D,
    VcomAndDataIntervalSetting = 0x50,
    LowerPowerDetection = 0x51,
    ResolutionSetting = 0x61,
    GateSourceStartSetting = 0x65,
    Revision = 0x70,
    AutoMeasureVcom = 0x80,
    VcomValue = 0x81,
    VcmDcSetting = 0x82,
    PartialWindow = 0x83,
    ProgramMode = 0x90,
    ActiveProgram = 0x91,
    ReadMtpData = 0x92,
    Revision2 = 0x9E,
    ReadMtpReservedBytes = 0x9F,
    UnknownInit5 = 0xA5,
    UnknownInit3 = 0xE0,
    PowerSaving = 0xE3,
    UnknownInit4 = 0xE6,
    UnkonwnInit2 = 0xE9,
    LvdVoltageSelect = 0xE4,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}
