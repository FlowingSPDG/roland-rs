//! Value types for V-60HD LAN / RS-232 parameters.

use crate::RolandError;

/// Cross-point channel (0–7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Channel {
    /// SDI IN 1
    Sdi1 = 0,
    /// SDI IN 2
    Sdi2 = 1,
    /// SDI IN 3
    Sdi3 = 2,
    /// SDI IN 4
    Sdi4 = 3,
    /// HDMI IN 5
    Hdmi5 = 4,
    /// HDMI/RGB IN 6
    HdmiRgb6 = 5,
    /// STILL/BKG IN 7
    Still7 = 6,
    /// STILL/BKG IN 8
    Still8 = 7,
}

impl Channel {
    /// Wire value 0–7.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Parse a wire value 0–7.
    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Sdi1),
            1 => Ok(Self::Sdi2),
            2 => Ok(Self::Sdi3),
            3 => Ok(Self::Sdi4),
            4 => Ok(Self::Hdmi5),
            5 => Ok(Self::HdmiRgb6),
            6 => Ok(Self::Still7),
            7 => Ok(Self::Still8),
            _ => Err(RolandError::OutOfRange),
        }
    }

    /// SDI IN 1–4.
    pub fn sdi(n: u8) -> Result<Self, RolandError> {
        match n {
            1 => Ok(Self::Sdi1),
            2 => Ok(Self::Sdi2),
            3 => Ok(Self::Sdi3),
            4 => Ok(Self::Sdi4),
            _ => Err(RolandError::OutOfRange),
        }
    }

    /// STILL/BKG IN 7–8.
    pub fn still(n: u8) -> Result<Self, RolandError> {
        match n {
            7 => Ok(Self::Still7),
            8 => Ok(Self::Still8),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// Mix / wipe transition type (`TRS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Transition {
    Mix = 0,
    Wipe1 = 1,
    Wipe2 = 2,
}

impl Transition {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Mix),
            1 => Ok(Self::Wipe1),
            2 => Ok(Self::Wipe2),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// Video transition time in 0.1 s steps (`TIM`, 0.0–4.0 s).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransitionTime(u8);

impl TransitionTime {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 40;

    pub fn new(tenths: u8) -> Result<Self, RolandError> {
        if tenths <= Self::MAX {
            Ok(Self(tenths))
        } else {
            Err(RolandError::OutOfRange)
        }
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

/// Output connector bus assignment (`OS1` / `OS2` / `OH1` / `OH2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OutputBus {
    Program = 0,
    Preview = 1,
    Aux = 2,
}

impl OutputBus {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Program),
            1 => Ok(Self::Preview),
            2 => Ok(Self::Aux),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// Tally lamp color (`TLY`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TallyColor {
    Dark = 0,
    Red = 1,
    Green = 2,
}

impl TallyColor {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Dark),
            1 => Ok(Self::Red),
            2 => Ok(Self::Green),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub const fn is_program(self) -> bool {
        matches!(self, Self::Red)
    }

    pub const fn is_preview(self) -> bool {
        matches!(self, Self::Green)
    }
}

/// Channel 6 connector select (`IPS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Channel6Input {
    Hdmi = 0,
    RgbComponent = 1,
}

impl Channel6Input {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Hdmi),
            1 => Ok(Self::RgbComponent),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// Input audio source for `IAL` / `IAM` / `IAS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AudioInput {
    AudioIn1 = 0,
    AudioIn2 = 1,
    AudioIn3 = 2,
    AudioIn4 = 3,
    AudioIn56 = 4,
    Sdi1 = 5,
    Sdi2 = 6,
    Sdi3 = 7,
    Sdi4 = 8,
    Hdmi5 = 9,
    Hdmi6 = 10,
}

impl AudioInput {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::AudioIn1),
            1 => Ok(Self::AudioIn2),
            2 => Ok(Self::AudioIn3),
            3 => Ok(Self::AudioIn4),
            4 => Ok(Self::AudioIn56),
            5 => Ok(Self::Sdi1),
            6 => Ok(Self::Sdi2),
            7 => Ok(Self::Sdi3),
            8 => Ok(Self::Sdi4),
            9 => Ok(Self::Hdmi5),
            10 => Ok(Self::Hdmi6),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// Analog audio input for delay (`ADT`, AUDIO IN 1–4 and 5/6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AnalogAudioInput {
    AudioIn1 = 0,
    AudioIn2 = 1,
    AudioIn3 = 2,
    AudioIn4 = 3,
    AudioIn56 = 4,
}

impl AnalogAudioInput {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::AudioIn1),
            1 => Ok(Self::AudioIn2),
            2 => Ok(Self::AudioIn3),
            3 => Ok(Self::AudioIn4),
            4 => Ok(Self::AudioIn56),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// `QAL` selector (single channel or all).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AudioLevelQuery {
    AudioIn1 = 0,
    AudioIn2 = 1,
    AudioIn3 = 2,
    AudioIn4 = 3,
    AudioIn56 = 4,
    Sdi1 = 5,
    Sdi2 = 6,
    Sdi3 = 7,
    Sdi4 = 8,
    Hdmi5 = 9,
    Hdmi6 = 10,
    MasterOut = 11,
    Aux = 12,
    All = 13,
}

impl AudioLevelQuery {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::AudioIn1),
            1 => Ok(Self::AudioIn2),
            2 => Ok(Self::AudioIn3),
            3 => Ok(Self::AudioIn4),
            4 => Ok(Self::AudioIn56),
            5 => Ok(Self::Sdi1),
            6 => Ok(Self::Sdi2),
            7 => Ok(Self::Sdi3),
            8 => Ok(Self::Sdi4),
            9 => Ok(Self::Hdmi5),
            10 => Ok(Self::Hdmi6),
            11 => Ok(Self::MasterOut),
            12 => Ok(Self::Aux),
            13 => Ok(Self::All),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// Audio level in 0.1 dB steps (`-800` = -80.0 dB … `100` = +10.0 dB, `-801` = -INF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioLevel(i16);

impl AudioLevel {
    pub const NEG_INF: Self = Self(-801);
    pub const MIN_TENTHS: i16 = -800;
    pub const MAX_TENTHS: i16 = 100;

    pub const fn neg_inf() -> Self {
        Self::NEG_INF
    }

    /// Level in tenths of a dB, or `-801` for -INF.
    pub fn from_tenths(tenths: i16) -> Result<Self, RolandError> {
        if tenths == -801 || (Self::MIN_TENTHS..=Self::MAX_TENTHS).contains(&tenths) {
            Ok(Self(tenths))
        } else {
            Err(RolandError::OutOfRange)
        }
    }

    pub const fn as_i16(self) -> i16 {
        self.0
    }
}

/// Input audio delay in 0.1 frame steps (`ADT`, 0.0–12.0 fps).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioDelay(u8);

impl AudioDelay {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 120;

    pub fn new(tenths: u8) -> Result<Self, RolandError> {
        if tenths <= Self::MAX {
            Ok(Self(tenths))
        } else {
            Err(RolandError::OutOfRange)
        }
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

/// PinP inset position (`PP1` / `PP2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PinPPosition {
    /// Horizontal (`-450`..=`450`)
    pub h: i16,
    /// Vertical (`-400`..=`400`)
    pub v: i16,
}

impl PinPPosition {
    pub fn new(h: i16, v: i16) -> Result<Self, RolandError> {
        if (-450..=450).contains(&h) && (-400..=400).contains(&v) {
            Ok(Self { h, v })
        } else {
            Err(RolandError::OutOfRange)
        }
    }
}

/// Split composition positions (`SPT`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SplitPosition {
    pub a: i16,
    pub b: i16,
}

impl SplitPosition {
    pub fn new(a: i16, b: i16) -> Result<Self, RolandError> {
        if (-250..=250).contains(&a) && (-250..=250).contains(&b) {
            Ok(Self { a, b })
        } else {
            Err(RolandError::OutOfRange)
        }
    }
}

/// Preset memory 1–8 (`MEM` wire value 0–7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemorySlot(u8);

impl MemorySlot {
    pub fn new(slot: u8) -> Result<Self, RolandError> {
        match slot {
            1..=8 => Ok(Self(slot - 1)),
            _ => Err(RolandError::OutOfRange),
        }
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

/// HDCP on/off (`HCP`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Hdcp {
    Off = 0,
    On = 1,
}

impl Hdcp {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::On),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// Test pattern (`TPT`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TestPattern {
    Off = 0,
    ColorBar75 = 1,
    ColorBar100 = 2,
    Ramp = 3,
    Step = 4,
    Hatch = 5,
}

impl TestPattern {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::ColorBar75),
            2 => Ok(Self::ColorBar100),
            3 => Ok(Self::Ramp),
            4 => Ok(Self::Step),
            5 => Ok(Self::Hatch),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// Test tone (`TTN`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TestTone {
    Off = 0,
    Hz1kNeg20 = 1,
    Hz1kNeg10 = 2,
    Hz1k0 = 3,
    Hz400Neg20 = 4,
    Hz400Neg10 = 5,
    Hz400Zero = 6,
}

impl TestTone {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::Hz1kNeg20),
            2 => Ok(Self::Hz1kNeg10),
            3 => Ok(Self::Hz1k0),
            4 => Ok(Self::Hz400Neg20),
            5 => Ok(Self::Hz400Neg10),
            6 => Ok(Self::Hz400Zero),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// `QPL` selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PanelQuery {
    Pgm = 0,
    Pst = 1,
    Aux = 2,
    PinPSplit = 3,
    Dsk = 4,
    OutputFade = 5,
    VideoFadeLevel = 6,
    All = 7,
}

impl PanelQuery {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Pgm),
            1 => Ok(Self::Pst),
            2 => Ok(Self::Aux),
            3 => Ok(Self::PinPSplit),
            4 => Ok(Self::Dsk),
            5 => Ok(Self::OutputFade),
            6 => Ok(Self::VideoFadeLevel),
            7 => Ok(Self::All),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// PinP / SPLIT field from `QPL` (Companion `buttonSet[3]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Composition {
    Off = 0,
    PinP1 = 1,
    PinP2 = 2,
    Split = 3,
}

impl Composition {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self, RolandError> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::PinP1),
            2 => Ok(Self::PinP2),
            3 => Ok(Self::Split),
            _ => Err(RolandError::OutOfRange),
        }
    }
}

/// Parsed `QPL:7` (ALL) panel snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PanelStatus {
    pub pgm: Channel,
    pub pst: Channel,
    pub aux: Channel,
    pub composition: Composition,
    pub dsk: bool,
    pub output_fade: bool,
    /// Video fader level when the device returns a 7th field (fw 1.11+).
    pub video_fade_level: Option<i32>,
}

impl PanelStatus {
    pub fn from_qpl_all(values: &[i32]) -> Result<Self, RolandError> {
        if values.len() < 6 {
            return Err(RolandError::InvalidResponse);
        }
        let pgm = Channel::from_u8(u8_from_i32(values[0])?)?;
        let pst = Channel::from_u8(u8_from_i32(values[1])?)?;
        let aux = Channel::from_u8(u8_from_i32(values[2])?)?;
        let composition = Composition::from_u8(u8_from_i32(values[3])?)?;
        let dsk = values[4] == 1;
        let output_fade = values[5] == 1;
        let video_fade_level = values.get(6).copied();
        Ok(Self {
            pgm,
            pst,
            aux,
            composition,
            dsk,
            output_fade,
            video_fade_level,
        })
    }
}

fn u8_from_i32(value: i32) -> Result<u8, RolandError> {
    u8::try_from(value).map_err(|_| RolandError::InvalidResponse)
}
