//! Roland V-60HD LAN / RS-232 protocol.
//!
//! Unlike VR-6HD / V-160HD (DTH/RQH), V-60HD uses 3-letter opcodes framed with
//! **STX (0x02)** and a trailing `;`. STX is required on LAN as well as RS-232.
//! After each command the controller must wait for ACK (`0x06`) before sending
//! the next one; some queries (notably `VER`) may omit ACK and return a `;`
//! payload only.
//!
//! Command tables follow the official *V-60HD Reference Manual* (LAN/RS-232)
//! and [companion-module-roland-v60hd](https://github.com/bitfocus/companion-module-roland-v60hd).

mod command;
mod parse;
mod types;

pub use command::*;
pub use parse::*;
pub use types::*;

/// Default TCP port for V-60HD LAN CONTROL (Telnet-style).
pub const TELNET_PORT: u16 = 8023;

/// Product name returned by `VER`.
pub const PRODUCT_NAME: &str = "V-60HD";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RolandError;
    use alloc::string::String;

    fn stx(body: &str) -> String {
        alloc::format!("\x02{body}")
    }

    #[test]
    fn encode_table_matches_reference() {
        let cases: &[(&str, Command)] = &[
            ("PGM:0;", pgm(Channel::Sdi1)),
            ("PST:4;", pst(Channel::Hdmi5)),
            ("AUX:7;", aux(Channel::Still8)),
            ("TRS:1;", set_transition(Transition::Wipe1)),
            (
                "TIM:10;",
                set_transition_time(TransitionTime::new(10).unwrap()),
            ),
            ("CUT;", cut()),
            ("ATO;", auto()),
            ("P1S;", pinp1_sw()),
            ("P2S;", pinp2_sw()),
            ("SPS;", split_sw()),
            ("DSK;", dsk_sw()),
            ("DVW;", dsk_pvw()),
            ("ATM;", auto_mixing()),
            ("FDE;", output_fade()),
            (
                "PP1:-450,400;",
                set_pinp1_position(PinPPosition::new(-450, 400).unwrap()),
            ),
            (
                "PP2:0,0;",
                set_pinp2_position(PinPPosition::new(0, 0).unwrap()),
            ),
            (
                "SPT:-250,250;",
                set_split_position(SplitPosition::new(-250, 250).unwrap()),
            ),
            ("DSS:3;", set_dsk_source(Channel::Sdi4)),
            ("KYL:255;", set_dsk_key_level(255)),
            ("KYG:0;", set_dsk_key_gain(0)),
            ("IPS:1;", set_channel6_input(Channel6Input::RgbComponent)),
            ("OS1:0;", set_sdi1_bus(OutputBus::Program)),
            ("OS2:1;", set_sdi2_bus(OutputBus::Preview)),
            ("OH1:2;", set_hdmi1_bus(OutputBus::Aux)),
            ("OH2:0;", set_hdmi2_bus(OutputBus::Program)),
            (
                "IAL:0,-801;",
                set_input_audio_level(AudioInput::AudioIn1, AudioLevel::neg_inf()),
            ),
            (
                "OAL:0;",
                set_master_level(AudioLevel::from_tenths(0).unwrap()),
            ),
            (
                "OAX:100;",
                set_aux_level(AudioLevel::from_tenths(100).unwrap()),
            ),
            (
                "ADT:4,120;",
                set_input_audio_delay(AnalogAudioInput::AudioIn56, AudioDelay::new(120).unwrap()),
            ),
            ("IAM:10;", mute_input(AudioInput::Hdmi6)),
            ("IAS:5;", solo_input(AudioInput::Sdi1)),
            ("QAL:13;", qal(AudioLevelQuery::All)),
            ("HCP:1;", set_hdcp(Hdcp::On)),
            ("TPT:1;", set_test_pattern(TestPattern::ColorBar75)),
            ("TTN:0;", set_test_tone(TestTone::Off)),
            ("MEM:0;", load_memory(MemorySlot::new(1).unwrap())),
            ("QPL:7;", qpl_all()),
            ("TLY;", tly()),
            ("ACS;", acs()),
            ("VER;", ver()),
        ];

        for (body, cmd) in cases {
            assert_eq!(cmd.encode_body(), *body, "body {body}");
            assert_eq!(cmd.encode(), stx(body), "wire {body}");
            let bytes = cmd.encode_bytes();
            assert_eq!(bytes[0], STX, "STX {body}");
            assert_eq!(*bytes.last().unwrap(), b';', "semicolon {body}");
        }
    }

    #[test]
    fn out_of_range_constructors() {
        assert_eq!(Channel::sdi(5), Err(RolandError::OutOfRange));
        assert_eq!(Channel::from_u8(8), Err(RolandError::OutOfRange));
        assert_eq!(TransitionTime::new(41), Err(RolandError::OutOfRange));
        assert_eq!(AudioLevel::from_tenths(-802), Err(RolandError::OutOfRange));
        assert_eq!(AudioLevel::from_tenths(101), Err(RolandError::OutOfRange));
        assert_eq!(PinPPosition::new(-451, 0), Err(RolandError::OutOfRange));
        assert_eq!(SplitPosition::new(0, 251), Err(RolandError::OutOfRange));
        assert_eq!(MemorySlot::new(0), Err(RolandError::OutOfRange));
        assert_eq!(MemorySlot::new(9), Err(RolandError::OutOfRange));
        assert_eq!(AudioDelay::new(121), Err(RolandError::OutOfRange));
    }

    #[test]
    fn queries_are_marked() {
        assert!(ver().is_query());
        assert!(tly().is_query());
        assert!(qpl_all().is_query());
        assert!(qal(AudioLevelQuery::MasterOut).is_query());
        assert!(acs().is_query());
        assert!(!cut().is_query());
        assert!(!pgm(Channel::Sdi1).is_query());
    }

    #[test]
    fn panel_status_from_qpl_all() {
        let status = PanelStatus::from_qpl_all(&[0, 1, 2, 3, 1, 0, 15]).unwrap();
        assert_eq!(status.pgm, Channel::Sdi1);
        assert_eq!(status.pst, Channel::Sdi2);
        assert_eq!(status.aux, Channel::Sdi3);
        assert_eq!(status.composition, Composition::Split);
        assert!(status.dsk);
        assert!(!status.output_fade);
        assert_eq!(status.video_fade_level, Some(15));
    }

    #[test]
    fn tally_color_bus_membership() {
        assert!(TallyColor::Red.is_program());
        assert!(!TallyColor::Red.is_preview());
        assert!(TallyColor::Green.is_preview());
        assert!(!TallyColor::Green.is_program());
        assert!(!TallyColor::Dark.is_program());
    }
}
