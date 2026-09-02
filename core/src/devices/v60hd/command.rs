//! Typed V-60HD command builders (STX is added at encode time).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use super::types::*;

/// A V-60HD control command (3-letter opcode plus optional decimal args).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    opcode: &'static str,
    args: Vec<i32>,
}

impl Command {
    fn simple(opcode: &'static str) -> Self {
        Self {
            opcode,
            args: Vec::new(),
        }
    }

    fn with_args(opcode: &'static str, args: Vec<i32>) -> Self {
        Self { opcode, args }
    }

    /// Escape hatch for a 3-letter opcode (used by hardware probes and tests).
    pub fn custom(opcode: &'static str, args: Vec<i32>) -> Result<Self, crate::RolandError> {
        if opcode.len() != 3 || !opcode.bytes().all(|b| b.is_ascii_alphabetic()) {
            return Err(crate::RolandError::InvalidValue);
        }
        Ok(Self { opcode, args })
    }

    /// Three-letter opcode (`PGM`, `CUT`, …).
    pub fn opcode(&self) -> &'static str {
        self.opcode
    }

    /// Decimal parameters in wire order.
    pub fn args(&self) -> &[i32] {
        &self.args
    }

    /// `true` for VER / TLY / QPL / QAL / ACS (expect a `;` payload, ACK optional).
    pub fn is_query(&self) -> bool {
        matches!(self.opcode, "VER" | "TLY" | "QPL" | "QAL" | "ACS")
    }

    /// Body without STX (`PGM:0;` or `CUT;`).
    pub fn encode_body(&self) -> String {
        if self.args.is_empty() {
            format!("{};", self.opcode)
        } else {
            let mut body = String::new();
            body.push_str(self.opcode);
            body.push(':');
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    body.push(',');
                }
                // `i32` Display is ASCII decimal, including a leading minus.
                body.push_str(&format!("{arg}"));
            }
            body.push(';');
            body
        }
    }

    /// Wire bytes as a string: STX (0x02) + body + `;` already in the body.
    pub fn encode(&self) -> String {
        format!("\x02{}", self.encode_body())
    }

    /// Encode to a byte vector (STX + ASCII body).
    pub fn encode_bytes(&self) -> Vec<u8> {
        self.encode().into_bytes()
    }
}

/// Select PGM channel.
pub fn pgm(channel: Channel) -> Command {
    Command::with_args("PGM", alloc::vec![i32::from(channel.as_u8())])
}

/// Select PST / preview channel.
pub fn pst(channel: Channel) -> Command {
    Command::with_args("PST", alloc::vec![i32::from(channel.as_u8())])
}

/// Select AUX channel.
pub fn aux(channel: Channel) -> Command {
    Command::with_args("AUX", alloc::vec![i32::from(channel.as_u8())])
}

/// Select mix / wipe transition.
pub fn set_transition(transition: Transition) -> Command {
    Command::with_args("TRS", alloc::vec![i32::from(transition.as_u8())])
}

/// Set video transition time.
pub fn set_transition_time(time: TransitionTime) -> Command {
    Command::with_args("TIM", alloc::vec![i32::from(time.as_u8())])
}

/// Press CUT.
pub fn cut() -> Command {
    Command::simple("CUT")
}

/// Press AUTO.
pub fn auto() -> Command {
    Command::simple("ATO")
}

/// Press PinP 1.
pub fn pinp1_sw() -> Command {
    Command::simple("P1S")
}

/// Press PinP 2.
pub fn pinp2_sw() -> Command {
    Command::simple("P2S")
}

/// Press SPLIT.
pub fn split_sw() -> Command {
    Command::simple("SPS")
}

/// Press DSK.
pub fn dsk_sw() -> Command {
    Command::simple("DSK")
}

/// Press DSK PVW.
pub fn dsk_pvw() -> Command {
    Command::simple("DVW")
}

/// Press AUTO MIXING.
pub fn auto_mixing() -> Command {
    Command::simple("ATM")
}

/// Press OUTPUT FADE.
pub fn output_fade() -> Command {
    Command::simple("FDE")
}

/// Set PinP 1 inset position.
pub fn set_pinp1_position(pos: PinPPosition) -> Command {
    Command::with_args("PP1", alloc::vec![i32::from(pos.h), i32::from(pos.v)])
}

/// Set PinP 2 inset position.
pub fn set_pinp2_position(pos: PinPPosition) -> Command {
    Command::with_args("PP2", alloc::vec![i32::from(pos.h), i32::from(pos.v)])
}

/// Set SPLIT composition positions.
pub fn set_split_position(pos: SplitPosition) -> Command {
    Command::with_args("SPT", alloc::vec![i32::from(pos.a), i32::from(pos.b)])
}

/// Set DSK source channel.
pub fn set_dsk_source(channel: Channel) -> Command {
    Command::with_args("DSS", alloc::vec![i32::from(channel.as_u8())])
}

/// Set DSK key level (0–255).
pub fn set_dsk_key_level(level: u8) -> Command {
    Command::with_args("KYL", alloc::vec![i32::from(level)])
}

/// Set DSK key gain (0–255).
pub fn set_dsk_key_gain(gain: u8) -> Command {
    Command::with_args("KYG", alloc::vec![i32::from(gain)])
}

/// Select HDMI vs RGB/Component for channel 6.
pub fn set_channel6_input(input: Channel6Input) -> Command {
    Command::with_args("IPS", alloc::vec![i32::from(input.as_u8())])
}

/// Assign SDI OUT 1 bus.
pub fn set_sdi1_bus(bus: OutputBus) -> Command {
    Command::with_args("OS1", alloc::vec![i32::from(bus.as_u8())])
}

/// Assign SDI OUT 2 bus.
pub fn set_sdi2_bus(bus: OutputBus) -> Command {
    Command::with_args("OS2", alloc::vec![i32::from(bus.as_u8())])
}

/// Assign HDMI OUT 1 bus.
pub fn set_hdmi1_bus(bus: OutputBus) -> Command {
    Command::with_args("OH1", alloc::vec![i32::from(bus.as_u8())])
}

/// Assign HDMI OUT 2 bus.
pub fn set_hdmi2_bus(bus: OutputBus) -> Command {
    Command::with_args("OH2", alloc::vec![i32::from(bus.as_u8())])
}

/// Set input audio level.
pub fn set_input_audio_level(input: AudioInput, level: AudioLevel) -> Command {
    Command::with_args(
        "IAL",
        alloc::vec![i32::from(input.as_u8()), i32::from(level.as_i16())],
    )
}

/// Set master output level.
pub fn set_master_level(level: AudioLevel) -> Command {
    Command::with_args("OAL", alloc::vec![i32::from(level.as_i16())])
}

/// Set AUX-bus audio level.
pub fn set_aux_level(level: AudioLevel) -> Command {
    Command::with_args("OAX", alloc::vec![i32::from(level.as_i16())])
}

/// Set analog input audio delay.
pub fn set_input_audio_delay(input: AnalogAudioInput, delay: AudioDelay) -> Command {
    Command::with_args(
        "ADT",
        alloc::vec![i32::from(input.as_u8()), i32::from(delay.as_u8())],
    )
}

/// Toggle mute on an input (`IAM`).
pub fn mute_input(input: AudioInput) -> Command {
    Command::with_args("IAM", alloc::vec![i32::from(input.as_u8())])
}

/// Toggle solo on an input (`IAS`).
pub fn solo_input(input: AudioInput) -> Command {
    Command::with_args("IAS", alloc::vec![i32::from(input.as_u8())])
}

/// Query audio level(s).
pub fn qal(query: AudioLevelQuery) -> Command {
    Command::with_args("QAL", alloc::vec![i32::from(query.as_u8())])
}

/// Set HDCP.
pub fn set_hdcp(hdcp: Hdcp) -> Command {
    Command::with_args("HCP", alloc::vec![i32::from(hdcp.as_u8())])
}

/// Set test pattern.
pub fn set_test_pattern(pattern: TestPattern) -> Command {
    Command::with_args("TPT", alloc::vec![i32::from(pattern.as_u8())])
}

/// Set test tone.
pub fn set_test_tone(tone: TestTone) -> Command {
    Command::with_args("TTN", alloc::vec![i32::from(tone.as_u8())])
}

/// Recall preset memory 1–8.
pub fn load_memory(slot: MemorySlot) -> Command {
    Command::with_args("MEM", alloc::vec![i32::from(slot.as_u8())])
}

/// Query panel button status.
pub fn qpl(query: PanelQuery) -> Command {
    Command::with_args("QPL", alloc::vec![i32::from(query.as_u8())])
}

/// Query all panel button fields (`QPL:7`).
pub fn qpl_all() -> Command {
    qpl(PanelQuery::All)
}

/// Query tally / cross-point lamps.
pub fn tly() -> Command {
    Command::simple("TLY")
}

/// Query device status (`ACS`).
///
/// Firmware 3.02 on LAN did not return ACK or a payload within 2s; treat as
/// optional and do not block a command queue on it.
pub fn acs() -> Command {
    Command::simple("ACS")
}

/// Query product / version (`VER`).
pub fn ver() -> Command {
    Command::simple("VER")
}
