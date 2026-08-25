//! Value types for V-160HD parameters.

use crate::RolandError;

/// HDMI / SDI / still / input-channel video source (0x00..=0x33).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoSource(u8);

impl VideoSource {
    pub const fn value(self) -> u8 {
        self.0
    }

    pub fn hdmi(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=8 => Ok(Self(n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub fn sdi(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=8 => Ok(Self(0x08 + n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub fn still(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=16 => Ok(Self(0x10 + n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    /// Crosspoint input 1–20 (`0x20..=0x33`).
    pub fn input(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=20 => Ok(Self(0x20 + n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        if value <= 0x33 {
            Ok(Self(value))
        } else {
            Err(RolandError::OutOfRange)
        }
    }
}

/// Physical input assignment (HDMI / SDI / still only, 0x00..=0x1F).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InputAssign {
    Hdmi1 = 0x00,
    Hdmi2 = 0x01,
    Hdmi3 = 0x02,
    Hdmi4 = 0x03,
    Hdmi5 = 0x04,
    Hdmi6 = 0x05,
    Hdmi7 = 0x06,
    Hdmi8 = 0x07,
    Sdi1 = 0x08,
    Sdi2 = 0x09,
    Sdi3 = 0x0A,
    Sdi4 = 0x0B,
    Sdi5 = 0x0C,
    Sdi6 = 0x0D,
    Sdi7 = 0x0E,
    Sdi8 = 0x0F,
    Still1 = 0x10,
    Still2 = 0x11,
    Still3 = 0x12,
    Still4 = 0x13,
    Still5 = 0x14,
    Still6 = 0x15,
    Still7 = 0x16,
    Still8 = 0x17,
    Still9 = 0x18,
    Still10 = 0x19,
    Still11 = 0x1A,
    Still12 = 0x1B,
    Still13 = 0x1C,
    Still14 = 0x1D,
    Still15 = 0x1E,
    Still16 = 0x1F,
}

impl InputAssign {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Crosspoint channel 1–10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputChannel(u8);

impl InputChannel {
    pub const CH1: Self = Self(0);
    pub const CH2: Self = Self(1);
    pub const CH3: Self = Self(2);
    pub const CH4: Self = Self(3);
    pub const CH5: Self = Self(4);
    pub const CH6: Self = Self(5);
    pub const CH7: Self = Self(6);
    pub const CH8: Self = Self(7);
    pub const CH9: Self = Self(8);
    pub const CH10: Self = Self(9);

    pub fn new(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=10 => Ok(Self(n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub const fn index(self) -> u8 {
        self.0
    }
}

/// Physical / bus outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Output {
    Hdmi1,
    Hdmi2,
    Hdmi3,
    Sdi1,
    Sdi2,
    Sdi3,
    Usb,
}

/// Output bus assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OutputAssign {
    Program = 0x00,
    SubProgram = 0x01,
    Preview = 0x02,
    Aux1 = 0x03,
    Aux2 = 0x04,
    Aux3 = 0x05,
    Dsk1Source = 0x06,
    Dsk2Source = 0x07,
    MultiView = 0x08,
    Input16View = 0x09,
    Still16View = 0x0A,
}

impl OutputAssign {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// AUX bus 1–3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuxBus {
    Aux1,
    Aux2,
    Aux3,
}

/// AUX PGM-link mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AuxLinkMode {
    Off = 0x00,
    AutoLink = 0x01,
    ManualLink = 0x02,
}

impl AuxLinkMode {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// On-air mix layers (program / sub-program PinP & DSK).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MixLayer {
    PgmPinP1 = 0x12,
    PgmPinP2 = 0x13,
    PgmPinP3 = 0x14,
    PgmPinP4 = 0x15,
    PgmDsk1 = 0x16,
    PgmDsk2 = 0x17,
    SubPgmPinP1 = 0x18,
    SubPgmPinP2 = 0x19,
    SubPgmPinP3 = 0x1A,
    SubPgmPinP4 = 0x1B,
    SubPgmDsk1 = 0x1C,
    SubPgmDsk2 = 0x1D,
}

impl MixLayer {
    pub const fn address_low(self) -> u8 {
        self as u8
    }
}

/// PinP & Key channel 1–4 (`0x1B..=0x1E`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PinPKey {
    Key1 = 0x1B,
    Key2 = 0x1C,
    Key3 = 0x1D,
    Key4 = 0x1E,
}

impl PinPKey {
    pub const fn mid(self) -> u8 {
        self as u8
    }

    pub fn from_index(n: u8) -> Result<Self, RolandError> {
        match n {
            1 => Ok(Self::Key1),
            2 => Ok(Self::Key2),
            3 => Ok(Self::Key3),
            4 => Ok(Self::Key4),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// DSK channel 1–2 (`0x1F..=0x20`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DskChannel {
    Dsk1 = 0x1F,
    Dsk2 = 0x20,
}

impl DskChannel {
    pub const fn mid(self) -> u8 {
        self as u8
    }

    pub fn from_index(n: u8) -> Result<Self, RolandError> {
        match n {
            1 => Ok(Self::Dsk1),
            2 => Ok(Self::Dsk2),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// PGM / PVW bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MixBus {
    Program = 0x00,
    Preview = 0x01,
}

impl MixBus {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// PinP / key type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PinPType {
    PinP = 0x00,
    LuminanceWhite = 0x01,
    LuminanceBlack = 0x02,
    Chroma = 0x03,
}

impl PinPType {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// DSK key type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DskType {
    LuminanceWhite = 0x00,
    LuminanceBlack = 0x01,
    Chroma = 0x02,
}

impl DskType {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// PinP window shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PinPShape {
    Rectangle = 0x00,
    Circle = 0x01,
    Diamond = 0x02,
}

impl PinPShape {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// PinP border color preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BorderColor {
    White = 0x00,
    Yellow = 0x01,
    Cyan = 0x02,
    Green = 0x03,
    Magenta = 0x04,
    Red = 0x05,
    Blue = 0x06,
    Black = 0x07,
    Custom = 0x08,
    SoftEdge = 0x09,
}

impl BorderColor {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Chroma key color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ChromaColor {
    Green = 0x00,
    Blue = 0x01,
}

impl ChromaColor {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Mix / wipe transition type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TransitionType {
    Mix = 0x00,
    Wipe = 0x01,
}

impl TransitionType {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Mix variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MixType {
    Mix = 0x00,
    Fam = 0x01,
    Nam = 0x02,
}

impl MixType {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Wipe pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WipeType {
    Horizontal = 0x00,
    Vertical = 0x01,
    UpperLeft = 0x02,
    UpperRight = 0x03,
    LowerLeft = 0x04,
    LowerRight = 0x05,
    HCenter = 0x06,
    VCenter = 0x07,
}

impl WipeType {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Wipe direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WipeDirection {
    Normal = 0x00,
    Reverse = 0x01,
    RoundTrip = 0x02,
}

impl WipeDirection {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Transition-time target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TransitionTime {
    MixWipe = 0x00,
    PinP1 = 0x01,
    PinP2 = 0x02,
    PinP3 = 0x03,
    PinP4 = 0x04,
    Dsk1 = 0x05,
    Dsk2 = 0x06,
    OutputFade = 0x07,
}

impl TransitionTime {
    pub const fn address_low(self) -> u8 {
        self as u8
    }
}

/// Preset memory 1–30.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemorySlot(u8);

impl MemorySlot {
    pub fn new(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=30 => Ok(Self(n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub const fn index(self) -> u8 {
        self.0
    }
}

/// Freeze type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FreezeType {
    All = 0x00,
    Select = 0x01,
}

impl FreezeType {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Freeze-select input (HDMI 1–8, SDI 1–8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FreezeInput(u8);

impl FreezeInput {
    pub fn hdmi(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=8 => Ok(Self(0x02 + n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub fn sdi(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=8 => Ok(Self(0x0A + n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub const fn address_low(self) -> u8 {
        self.0
    }
}

/// Camera 1–16 (`0x41..=0x50`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CameraId(u8);

impl CameraId {
    pub fn new(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=16 => Ok(Self(0x41 + n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub const fn mid(self) -> u8 {
        self.0
    }
}

/// Camera preset 1–10, or none (`0x7F`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CameraPreset(u8);

impl CameraPreset {
    pub const NONE: Self = Self(0x7F);

    pub fn new(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=10 => Ok(Self(n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Camera pan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PanDirection {
    Stop = 0x00,
    Right = 0x01,
    Left = 0x7F,
}

impl PanDirection {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Camera tilt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TiltDirection {
    Stop = 0x00,
    Up = 0x01,
    Down = 0x7F,
}

impl TiltDirection {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Camera zoom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ZoomCommand {
    Stop = 0x00,
    InSlow = 0x01,
    InFast = 0x02,
    OutFast = 0x7E,
    OutSlow = 0x7F,
}

impl ZoomCommand {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// Camera focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FocusCommand {
    Stop = 0x00,
    Far = 0x01,
    Near = 0x7F,
}

impl FocusCommand {
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// HDMI/SDI tally source index (0–15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TallySource(u8);

impl TallySource {
    pub fn hdmi(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=8 => Ok(Self(n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub fn sdi(n: u8) -> Result<Self, RolandError> {
        match n {
            1..=8 => Ok(Self(0x08 + n - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub const fn index(self) -> u8 {
        self.0
    }
}

/// Tally state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TallyState {
    Off = 0x00,
    Program = 0x01,
    Preview = 0x02,
}

impl TallyState {
    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::Program),
            2 => Ok(Self::Preview),
            _ => Err(RolandError::OutOfRange),
        }
    }
}
