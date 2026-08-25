//! Roland V-160HD address map and typed command builders.
//!
//! Addresses and value encodings follow the official *V-160HD Remote Control
//! Guide* and the Bitfocus Companion module `companion-module-roland-v160hd`.
//!
//! LAN control uses Telnet on TCP port **8023**, a 4-digit network password,
//! DTH/RQH frames, and LF-terminated lines.

mod address;
mod command;
mod types;

pub use address::*;
pub use command::*;
pub use types::*;

/// Default Telnet port for V-160HD LAN CONTROL.
pub const TELNET_PORT: u16 = 8023;

/// Product name returned by `VER;`.
pub const PRODUCT_NAME: &str = "V-160HD";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;

    #[test]
    fn pgm_select_hdmi3_matches_official_example() {
        let cmd = select_pgm(VideoSource::hdmi(3).unwrap());
        assert_eq!(cmd.encode(), "DTH:002100,02;");
    }

    #[test]
    fn cut_switch_press() {
        let cmd = press_switch(switch::CUT);
        assert_eq!(cmd.encode(), "DTH:0B001E,01;");
    }

    #[test]
    fn input1_assign_hdmi1() {
        let cmd = assign_input(InputChannel::CH1, InputAssign::Hdmi1);
        assert_eq!(cmd.encode(), "DTH:000000,00;");
    }

    #[test]
    fn usb_output_assign_program() {
        let cmd = assign_output(Output::Usb, OutputAssign::Program);
        assert_eq!(cmd.encode(), "DTH:000110,00;");
    }

    #[test]
    fn pinp_position_uses_two_14bit_writes() {
        let cmds = set_pinp_position_h(PinPKey::Key1, -1000);
        match cmds {
            [Command::WriteParameter {
                address: a0,
                value: v0,
            }, Command::WriteParameter {
                address: a1,
                value: v1,
            }] => {
                assert_eq!(a0.to_hex(), "001B04");
                assert_eq!(a1.to_hex(), "001B05");
                assert_eq!((v0, v1), crate::midi::encode_14bit(-1000));
            }
            _ => panic!("expected two write commands"),
        }
    }

    #[test]
    fn transition_time_mix_1s() {
        let cmd = set_transition_time(TransitionTime::MixWipe, 10);
        assert_eq!(cmd.encode(), "DTH:001700,0A;");
    }

    #[test]
    fn load_memory_1() {
        let cmd = load_memory(MemorySlot::new(1).unwrap());
        assert_eq!(cmd.encode(), "DTH:0A0000,00;");
    }

    #[test]
    fn camera1_pan_left() {
        let cmd = camera_pan(CameraId::new(1).unwrap(), PanDirection::Left);
        assert_eq!(cmd.encode(), "DTH:024122,7F;");
    }

    #[test]
    fn run_macro_1() {
        let cmd = run_macro(1).unwrap();
        assert_eq!(cmd.encode(), "DTH:500504,00;");
    }

    #[test]
    fn subscribe_tally_on() {
        let cmd = subscribe_tally(true);
        assert_eq!(cmd.encode(), "DTH:0C0100,01;");
    }

    #[test]
    fn tally_dump_request_is_16_bytes() {
        assert_eq!(read_tally_dump().encode(), "RQH:0C0000,000010;");
    }

    #[test]
    fn tally_updates_parses_dump() {
        let response = crate::Response::parse("DTH:0C0000,00010203;").unwrap();
        let updates = tally_updates(&response).unwrap();
        assert_eq!(updates[0], (0, TallyState::Off));
        assert_eq!(updates[1], (1, TallyState::Program));
        assert_eq!(updates[2], (2, TallyState::Preview));
        assert_eq!(updates[3], (3, TallyState::Both));
    }

    #[test]
    fn tally_state_bus_membership() {
        assert!(TallyState::Program.is_program());
        assert!(!TallyState::Program.is_preview());
        assert!(TallyState::Preview.is_preview());
        assert!(!TallyState::Preview.is_program());
        assert!(TallyState::Both.is_program());
        assert!(TallyState::Both.is_preview());
        assert!(!TallyState::Off.is_program());
        assert!(!TallyState::Off.is_preview());
    }
}
