//! SysEx addresses for V-160HD.

use super::{AuxBus, DskChannel, FreezeInput, InputChannel, Output, PinPKey, TallySource};
use crate::Address;

pub const fn input_assign(ch: InputChannel) -> Address {
    Address::new(0x00, 0x00, ch.index())
}

pub fn output_assign(output: Output) -> Address {
    match output {
        Output::Hdmi1 => Address::new(0x00, 0x00, 0x0A),
        Output::Hdmi2 => Address::new(0x00, 0x00, 0x0B),
        Output::Hdmi3 => Address::new(0x00, 0x00, 0x0C),
        Output::Sdi1 => Address::new(0x00, 0x00, 0x0D),
        Output::Sdi2 => Address::new(0x00, 0x00, 0x0E),
        Output::Sdi3 => Address::new(0x00, 0x00, 0x0F),
        Output::Usb => Address::new(0x00, 0x01, 0x10),
    }
}

pub const fn aux_source(aux: AuxBus) -> Address {
    match aux {
        AuxBus::Aux1 => Address::new(0x00, 0x00, 0x11),
        AuxBus::Aux2 => Address::new(0x00, 0x00, 0x2E),
        AuxBus::Aux3 => Address::new(0x00, 0x00, 0x2F),
    }
}

pub const fn aux_mute(aux: AuxBus) -> Address {
    match aux {
        AuxBus::Aux1 => Address::new(0x01, 0x22, 0x03),
        AuxBus::Aux2 => Address::new(0x01, 0x25, 0x03),
        AuxBus::Aux3 => Address::new(0x01, 0x26, 0x03),
    }
}

pub const AUX_LINK_MODE: Address = Address::new(0x02, 0x01, 0x0D);

pub const fn aux_linked(aux: AuxBus) -> Address {
    match aux {
        AuxBus::Aux1 => Address::new(0x02, 0x01, 0x54),
        AuxBus::Aux2 => Address::new(0x02, 0x01, 0x55),
        AuxBus::Aux3 => Address::new(0x02, 0x01, 0x56),
    }
}

pub const fn mix_layer(low: u8) -> Address {
    Address::new(0x00, 0x00, low)
}

pub const fn pinp_fade(key: PinPKey) -> Address {
    let low = match key {
        PinPKey::Key1 => 0x05,
        PinPKey::Key2 => 0x06,
        PinPKey::Key3 => 0x07,
        PinPKey::Key4 => 0x08,
    };
    Address::new(0x02, 0x03, low)
}

pub const fn pinp(key: PinPKey, low: u8) -> Address {
    Address::new(0x00, key.mid(), low)
}

pub const fn dsk(ch: DskChannel, low: u8) -> Address {
    Address::new(0x00, ch.mid(), low)
}

pub const fn transition_time(low: u8) -> Address {
    Address::new(0x00, 0x17, low)
}

pub const TRANSITION_TYPE: Address = Address::new(0x00, 0x18, 0x00);
pub const MIX_TYPE: Address = Address::new(0x00, 0x18, 0x01);
pub const WIPE_TYPE: Address = Address::new(0x00, 0x18, 0x02);
pub const WIPE_DIRECTION: Address = Address::new(0x00, 0x18, 0x03);

pub const PGM_SELECT: Address = Address::new(0x00, 0x21, 0x00);
pub const PST_SELECT: Address = Address::new(0x00, 0x21, 0x01);

pub const MEMORY_LOAD: Address = Address::new(0x0A, 0x00, 0x00);
pub const MEMORY_SAVE: Address = Address::new(0x0A, 0x00, 0x01);
pub const MEMORY_INIT: Address = Address::new(0x0A, 0x00, 0x02);
pub const MEMORY_LOADED: Address = Address::new(0x0A, 0x00, 0x03);

pub const fn memory_name(slot: u8, char_index: u8) -> Address {
    Address::new(0x60, slot, char_index)
}

pub const FREEZE: Address = Address::new(0x02, 0x05, 0x00);
pub const FREEZE_TYPE: Address = Address::new(0x02, 0x05, 0x01);

pub const fn freeze_select(input: FreezeInput) -> Address {
    Address::new(0x02, 0x05, input.address_low())
}

pub const TALLY_SUBSCRIBE: Address = Address::new(0x0C, 0x01, 0x00);

pub const fn tally(source: TallySource) -> Address {
    Address::new(0x0C, 0x00, source.index())
}

pub const MACRO_RUN: Address = Address::new(0x50, 0x05, 0x04);

pub const fn camera(mid: u8, low: u8) -> Address {
    Address::new(0x02, mid, low)
}

/// Panel-switch addresses (`0B 00 xx`), matching Companion `CHOICES_SWITCHES`.
pub mod switch {
    use crate::Address;

    pub const PGM_A_1: Address = Address::new(0x0B, 0x00, 0x00);
    pub const PGM_A_2: Address = Address::new(0x0B, 0x00, 0x01);
    pub const PGM_A_3: Address = Address::new(0x0B, 0x00, 0x02);
    pub const PGM_A_4: Address = Address::new(0x0B, 0x00, 0x03);
    pub const PGM_A_5: Address = Address::new(0x0B, 0x00, 0x04);
    pub const PGM_A_6: Address = Address::new(0x0B, 0x00, 0x05);
    pub const PGM_A_7: Address = Address::new(0x0B, 0x00, 0x06);
    pub const PGM_A_8: Address = Address::new(0x0B, 0x00, 0x07);
    pub const PGM_A_9: Address = Address::new(0x0B, 0x00, 0x08);
    pub const PGM_A_10: Address = Address::new(0x0B, 0x00, 0x09);
    pub const PST_B_1: Address = Address::new(0x0B, 0x00, 0x0A);
    pub const PST_B_2: Address = Address::new(0x0B, 0x00, 0x0B);
    pub const PST_B_3: Address = Address::new(0x0B, 0x00, 0x0C);
    pub const PST_B_4: Address = Address::new(0x0B, 0x00, 0x0D);
    pub const PST_B_5: Address = Address::new(0x0B, 0x00, 0x0E);
    pub const PST_B_6: Address = Address::new(0x0B, 0x00, 0x0F);
    pub const PST_B_7: Address = Address::new(0x0B, 0x00, 0x10);
    pub const PST_B_8: Address = Address::new(0x0B, 0x00, 0x11);
    pub const PST_B_9: Address = Address::new(0x0B, 0x00, 0x12);
    pub const PST_B_10: Address = Address::new(0x0B, 0x00, 0x13);
    pub const AUX_1: Address = Address::new(0x0B, 0x00, 0x14);
    pub const AUX_2: Address = Address::new(0x0B, 0x00, 0x15);
    pub const AUX_3: Address = Address::new(0x0B, 0x00, 0x16);
    pub const AUX_4: Address = Address::new(0x0B, 0x00, 0x17);
    pub const AUX_5: Address = Address::new(0x0B, 0x00, 0x18);
    pub const AUX_6: Address = Address::new(0x0B, 0x00, 0x19);
    pub const AUX_7: Address = Address::new(0x0B, 0x00, 0x1A);
    pub const AUX_8: Address = Address::new(0x0B, 0x00, 0x1B);
    pub const AUX_9: Address = Address::new(0x0B, 0x00, 0x1C);
    pub const AUX_10: Address = Address::new(0x0B, 0x00, 0x1D);
    pub const CUT: Address = Address::new(0x0B, 0x00, 0x1E);
    pub const AUTO: Address = Address::new(0x0B, 0x00, 0x1F);
    pub const TRANSITION: Address = Address::new(0x0B, 0x00, 0x20);
    pub const MODE: Address = Address::new(0x0B, 0x00, 0x21);
    pub const INPUT_ASSIGN: Address = Address::new(0x0B, 0x00, 0x22);
    pub const PGM_CENTER_ENCODER: Address = Address::new(0x0B, 0x00, 0x23);
    pub const PST_CENTER_ENCODER: Address = Address::new(0x0B, 0x00, 0x24);
    pub const SPLIT_A: Address = Address::new(0x0B, 0x00, 0x25);
    pub const SPLIT_B: Address = Address::new(0x0B, 0x00, 0x26);
    pub const AUTO_MIXING: Address = Address::new(0x0B, 0x00, 0x27);
    pub const CAPTURE: Address = Address::new(0x0B, 0x00, 0x28);
    pub const USER_1: Address = Address::new(0x0B, 0x00, 0x29);
    pub const USER_2: Address = Address::new(0x0B, 0x00, 0x2A);
    pub const USER_3: Address = Address::new(0x0B, 0x00, 0x2B);
    pub const USER_4: Address = Address::new(0x0B, 0x00, 0x2C);
    pub const PINP1_POS_H: Address = Address::new(0x0B, 0x00, 0x2D);
    pub const PINP1_POS_V: Address = Address::new(0x0B, 0x00, 0x2E);
    pub const PINP1_SOURCE: Address = Address::new(0x0B, 0x00, 0x2F);
    pub const PINP1_PVW: Address = Address::new(0x0B, 0x00, 0x30);
    pub const PINP1_PGM: Address = Address::new(0x0B, 0x00, 0x31);
    pub const PINP2_POS_H: Address = Address::new(0x0B, 0x00, 0x32);
    pub const PINP2_POS_V: Address = Address::new(0x0B, 0x00, 0x33);
    pub const PINP2_SOURCE: Address = Address::new(0x0B, 0x00, 0x34);
    pub const PINP2_PVW: Address = Address::new(0x0B, 0x00, 0x35);
    pub const PINP2_PGM: Address = Address::new(0x0B, 0x00, 0x36);
    pub const PINP3_POS_H: Address = Address::new(0x0B, 0x00, 0x37);
    pub const PINP3_POS_V: Address = Address::new(0x0B, 0x00, 0x38);
    pub const PINP3_SOURCE: Address = Address::new(0x0B, 0x00, 0x39);
    pub const PINP3_PVW: Address = Address::new(0x0B, 0x00, 0x3A);
    pub const PINP3_PGM: Address = Address::new(0x0B, 0x00, 0x3B);
    pub const PINP4_POS_H: Address = Address::new(0x0B, 0x00, 0x3C);
    pub const PINP4_POS_V: Address = Address::new(0x0B, 0x00, 0x3D);
    pub const PINP4_SOURCE: Address = Address::new(0x0B, 0x00, 0x3E);
    pub const PINP4_PVW: Address = Address::new(0x0B, 0x00, 0x3F);
    pub const PINP4_PGM: Address = Address::new(0x0B, 0x00, 0x40);
    pub const DSK1_SOURCE: Address = Address::new(0x0B, 0x00, 0x41);
    pub const DSK1_PVW: Address = Address::new(0x0B, 0x00, 0x42);
    pub const DSK1_PGM: Address = Address::new(0x0B, 0x00, 0x43);
    pub const DSK2_SOURCE: Address = Address::new(0x0B, 0x00, 0x44);
    pub const DSK2_PVW: Address = Address::new(0x0B, 0x00, 0x45);
    pub const DSK2_PGM: Address = Address::new(0x0B, 0x00, 0x46);
    pub const MONITOR_1: Address = Address::new(0x0B, 0x00, 0x47);
    pub const MONITOR_2: Address = Address::new(0x0B, 0x00, 0x48);
    pub const MONITOR_3: Address = Address::new(0x0B, 0x00, 0x49);
    pub const MONITOR_4: Address = Address::new(0x0B, 0x00, 0x4A);
    pub const MENU: Address = Address::new(0x0B, 0x00, 0x4B);
    pub const EXIT: Address = Address::new(0x0B, 0x00, 0x4C);
    pub const ENTER: Address = Address::new(0x0B, 0x00, 0x4D);
    pub const OUTPUT_FADE: Address = Address::new(0x0B, 0x00, 0x4E);
    pub const SEQUENCER_ON: Address = Address::new(0x0B, 0x00, 0x4F);
    pub const SEQUENCER_AUTO: Address = Address::new(0x0B, 0x00, 0x50);
    pub const SEQUENCER_PREV: Address = Address::new(0x0B, 0x00, 0x51);
    pub const SEQUENCER_NEXT: Address = Address::new(0x0B, 0x00, 0x52);
}
