//! Typed DTH/RQH command builders for V-160HD.

use super::address::{self, switch};
use super::types::*;
use crate::midi::{encode_14bit, encode_s7};
use crate::{Address, Command, RolandError};

fn write(address: Address, value: u8) -> Command {
    Command::WriteParameter { address, value }
}

fn write_14(base: Address, scaled: i32) -> [Command; 2] {
    let (msb, lsb) = encode_14bit(scaled);
    [write(base, msb), write(base.offset_low(1), lsb)]
}

fn on_off(on: bool) -> u8 {
    if on {
        1
    } else {
        0
    }
}

/// Assign a physical source to input 1–10.
pub fn assign_input(ch: InputChannel, source: InputAssign) -> Command {
    write(address::input_assign(ch), source.value())
}

/// Assign a bus to an output connector.
pub fn assign_output(output: Output, assign: OutputAssign) -> Command {
    write(address::output_assign(output), assign.value())
}

/// Set AUX PGM-link mode.
pub fn set_aux_link_mode(mode: AuxLinkMode) -> Command {
    write(address::AUX_LINK_MODE, mode.value())
}

/// Enable or disable AUX PGM link for one AUX bus.
pub fn set_aux_linked(aux: AuxBus, on: bool) -> Command {
    write(address::aux_linked(aux), on_off(on))
}

/// Set AUX source.
pub fn assign_aux(aux: AuxBus, source: VideoSource) -> Command {
    write(address::aux_source(aux), source.value())
}

/// Mute or unmute AUX audio.
pub fn mute_aux(aux: AuxBus, muted: bool) -> Command {
    write(address::aux_mute(aux), on_off(muted))
}

/// Enable or disable a PGM / SUB PGM PinP or DSK layer.
pub fn set_layer_enable(layer: MixLayer, enable: bool) -> Command {
    write(address::mix_layer(layer.address_low()), on_off(enable))
}

/// Enable or disable PinP fade.
pub fn set_pinp_fade(key: PinPKey, enable: bool) -> Command {
    write(address::pinp_fade(key), on_off(enable))
}

/// Turn a PinP key on or off on PGM or PVW.
pub fn set_pinp_bus(key: PinPKey, bus: MixBus, on: bool) -> Command {
    write(address::pinp(key, bus.value()), on_off(on))
}

/// Set PinP source.
pub fn set_pinp_source(key: PinPKey, source: VideoSource) -> Command {
    write(address::pinp(key, 0x02), source.value())
}

/// Set PinP type.
pub fn set_pinp_type(key: PinPKey, ty: PinPType) -> Command {
    write(address::pinp(key, 0x03), ty.value())
}

/// Horizontal PinP position. `scaled` is value × 10 (range `-1000..=1000`).
pub fn set_pinp_position_h(key: PinPKey, scaled: i32) -> [Command; 2] {
    write_14(address::pinp(key, 0x04), scaled)
}

/// Vertical PinP position. `scaled` is value × 10 (range `-1000..=1000`).
pub fn set_pinp_position_v(key: PinPKey, scaled: i32) -> [Command; 2] {
    write_14(address::pinp(key, 0x06), scaled)
}

/// PinP size. `scaled` is value × 10 (range `100..=1000` for 10.0–100.0%).
pub fn set_pinp_size(key: PinPKey, scaled: i32) -> [Command; 2] {
    write_14(address::pinp(key, 0x08), scaled)
}

/// Horizontal crop. `scaled` is value × 10 (range `0..=1000`).
pub fn set_pinp_crop_h(key: PinPKey, scaled: i32) -> [Command; 2] {
    write_14(address::pinp(key, 0x0A), scaled)
}

/// Vertical crop. `scaled` is value × 10 (range `0..=1000`).
pub fn set_pinp_crop_v(key: PinPKey, scaled: i32) -> [Command; 2] {
    write_14(address::pinp(key, 0x0C), scaled)
}

/// PinP window shape.
pub fn set_pinp_shape(key: PinPKey, shape: PinPShape) -> Command {
    write(address::pinp(key, 0x0E), shape.value())
}

/// PinP border color preset.
pub fn set_pinp_border_color(key: PinPKey, color: BorderColor) -> Command {
    write(address::pinp(key, 0x0F), color.value())
}

/// PinP border width (0–14).
pub fn set_pinp_border_width(key: PinPKey, width: u8) -> Command {
    write(address::pinp(key, 0x10), width)
}

/// View position H. `scaled` is value × 10 (range `-500..=500`).
pub fn set_pinp_view_position_h(key: PinPKey, scaled: i32) -> [Command; 2] {
    write_14(address::pinp(key, 0x11), scaled)
}

/// View position V. `scaled` is value × 10 (range `-500..=500`).
pub fn set_pinp_view_position_v(key: PinPKey, scaled: i32) -> [Command; 2] {
    write_14(address::pinp(key, 0x13), scaled)
}

/// View zoom percent (100–400).
pub fn set_pinp_view_zoom(key: PinPKey, percent: u16) -> [Command; 2] {
    write_14(address::pinp(key, 0x15), percent as i32)
}

/// Key level (0–255).
pub fn set_pinp_key_level(key: PinPKey, level: u8) -> [Command; 2] {
    write_14(address::pinp(key, 0x17), level as i32)
}

/// Key gain (0–255).
pub fn set_pinp_key_gain(key: PinPKey, gain: u8) -> [Command; 2] {
    write_14(address::pinp(key, 0x19), gain as i32)
}

/// Mix level (0–255).
pub fn set_pinp_mix_level(key: PinPKey, level: u8) -> [Command; 2] {
    write_14(address::pinp(key, 0x1B), level as i32)
}

/// Chroma key color.
pub fn set_pinp_chroma_color(key: PinPKey, color: ChromaColor) -> Command {
    write(address::pinp(key, 0x1D), color.value())
}

/// Hue width (`-30..=30`), encoded as signed 7-bit.
pub fn set_pinp_hue_width(key: PinPKey, width: i8) -> Command {
    write(address::pinp(key, 0x1E), encode_s7(width))
}

/// Hue fine (0–360).
pub fn set_pinp_hue_fine(key: PinPKey, degrees: u16) -> [Command; 2] {
    write_14(address::pinp(key, 0x1F), degrees as i32)
}

/// Saturation width (`-127..=127`).
pub fn set_pinp_saturation_width(key: PinPKey, width: i16) -> [Command; 2] {
    write_14(address::pinp(key, 0x21), width as i32)
}

/// Saturation fine (0–255).
pub fn set_pinp_saturation_fine(key: PinPKey, fine: u8) -> [Command; 2] {
    write_14(address::pinp(key, 0x23), fine as i32)
}

/// Custom border red (0–255).
pub fn set_pinp_border_red(key: PinPKey, red: u8) -> [Command; 2] {
    write_14(address::pinp(key, 0x25), red as i32)
}

/// Custom border green (0–255).
pub fn set_pinp_border_green(key: PinPKey, green: u8) -> [Command; 2] {
    write_14(address::pinp(key, 0x27), green as i32)
}

/// Custom border blue (0–255).
pub fn set_pinp_border_blue(key: PinPKey, blue: u8) -> [Command; 2] {
    write_14(address::pinp(key, 0x29), blue as i32)
}

/// Turn DSK on or off on PGM or PVW.
pub fn set_dsk_bus(ch: DskChannel, bus: MixBus, on: bool) -> Command {
    write(address::dsk(ch, bus.value()), on_off(on))
}

/// DSK key source.
pub fn set_dsk_key_source(ch: DskChannel, source: VideoSource) -> Command {
    write(address::dsk(ch, 0x03), source.value())
}

/// DSK fill source.
pub fn set_dsk_fill_source(ch: DskChannel, source: VideoSource) -> Command {
    write(address::dsk(ch, 0x04), source.value())
}

/// DSK key type.
pub fn set_dsk_type(ch: DskChannel, ty: DskType) -> Command {
    write(address::dsk(ch, 0x05), ty.value())
}

/// Transition time in tenths of a second (`0..=40` for 0.0–4.0 s).
pub fn set_transition_time(kind: TransitionTime, tenths: u8) -> Command {
    write(address::transition_time(kind.address_low()), tenths)
}

/// Mix vs wipe.
pub fn set_transition_type(ty: TransitionType) -> Command {
    write(address::TRANSITION_TYPE, ty.value())
}

/// Mix variant.
pub fn set_mix_type(ty: MixType) -> Command {
    write(address::MIX_TYPE, ty.value())
}

/// Wipe pattern.
pub fn set_wipe_type(ty: WipeType) -> Command {
    write(address::WIPE_TYPE, ty.value())
}

/// Wipe direction.
pub fn set_wipe_direction(dir: WipeDirection) -> Command {
    write(address::WIPE_DIRECTION, dir.value())
}

/// Press a panel switch (value `01`).
pub fn press_switch(sw: Address) -> Command {
    write(sw, 0x01)
}

/// Release a panel switch (value `00`).
pub fn release_switch(sw: Address) -> Command {
    write(sw, 0x00)
}

/// CUT switch press.
pub fn cut() -> Command {
    press_switch(switch::CUT)
}

/// AUTO switch press.
pub fn auto_transition() -> Command {
    press_switch(switch::AUTO)
}

/// Select PGM source.
pub fn select_pgm(source: VideoSource) -> Command {
    write(address::PGM_SELECT, source.value())
}

/// Select PST / PVW source.
pub fn select_pst(source: VideoSource) -> Command {
    write(address::PST_SELECT, source.value())
}

/// Load preset memory.
pub fn load_memory(slot: MemorySlot) -> Command {
    write(address::MEMORY_LOAD, slot.index())
}

/// Save preset memory.
pub fn save_memory(slot: MemorySlot) -> Command {
    write(address::MEMORY_SAVE, slot.index())
}

/// Initialize preset memory.
pub fn init_memory(slot: MemorySlot) -> Command {
    write(address::MEMORY_INIT, slot.index())
}

/// Read last loaded memory number.
pub fn read_loaded_memory() -> Command {
    Command::ReadParameter {
        address: address::MEMORY_LOADED,
        size: 1,
    }
}

/// Freeze on/off.
pub fn set_freeze(on: bool) -> Command {
    write(address::FREEZE, on_off(on))
}

/// Freeze type.
pub fn set_freeze_type(ty: FreezeType) -> Command {
    write(address::FREEZE_TYPE, ty.value())
}

/// Enable freeze for a selected input.
pub fn set_freeze_select(input: FreezeInput, enable: bool) -> Command {
    write(address::freeze_select(input), on_off(enable))
}

/// Subscribe to tally notifications.
pub fn subscribe_tally(enable: bool) -> Command {
    write(address::TALLY_SUBSCRIBE, on_off(enable))
}

/// Read tally for one HDMI/SDI input.
pub fn read_tally(source: TallySource) -> Command {
    Command::ReadParameter {
        address: address::tally(source),
        size: 1,
    }
}

/// Read the 16-byte HDMI/SDI tally dump (`0C0000`).
pub fn read_tally_dump() -> Command {
    Command::ReadParameter {
        address: Address::new(0x0C, 0x00, 0x00),
        size: 16,
    }
}

/// Parse a tally DTH notification or dump into `(source_index, state)` pairs.
pub fn tally_updates(response: &crate::Response) -> Option<alloc::vec::Vec<(u8, TallyState)>> {
    use crate::Response;
    match response {
        Response::Data { address, value } if address.high == 0x0C && address.mid == 0x00 => Some(
            alloc::vec![(address.low, TallyState::from_u8(*value).ok()?)],
        ),
        Response::DataBlock { address, bytes } if address.high == 0x0C && address.mid == 0x00 => {
            Some(
                bytes
                    .iter()
                    .enumerate()
                    .filter_map(|(i, b)| {
                        let idx = address.low.wrapping_add(i as u8);
                        TallyState::from_u8(*b).ok().map(|state| (idx, state))
                    })
                    .collect(),
            )
        }
        _ => None,
    }
}

/// Run macro 1–100.
pub fn run_macro(n: u8) -> Result<Command, RolandError> {
    match n {
        1..=100 => Ok(write(address::MACRO_RUN, n - 1)),
        _ => Err(RolandError::OutOfRange),
    }
}

/// Recall camera preset.
pub fn camera_preset(cam: CameraId, preset: CameraPreset) -> Command {
    write(address::camera(cam.mid(), 0x21), preset.value())
}

/// Camera pan.
pub fn camera_pan(cam: CameraId, dir: PanDirection) -> Command {
    write(address::camera(cam.mid(), 0x22), dir.value())
}

/// Camera tilt.
pub fn camera_tilt(cam: CameraId, dir: TiltDirection) -> Command {
    write(address::camera(cam.mid(), 0x23), dir.value())
}

/// Pan/tilt speed (0–24).
pub fn camera_pt_speed(cam: CameraId, speed: u8) -> Command {
    write(address::camera(cam.mid(), 0x24), speed)
}

/// Camera zoom.
pub fn camera_zoom(cam: CameraId, cmd: ZoomCommand) -> Command {
    write(address::camera(cam.mid(), 0x25), cmd.value())
}

/// Camera focus.
pub fn camera_focus(cam: CameraId, cmd: FocusCommand) -> Command {
    write(address::camera(cam.mid(), 0x26), cmd.value())
}

/// Auto focus.
pub fn camera_auto_focus(cam: CameraId, on: bool) -> Command {
    write(address::camera(cam.mid(), 0x27), on_off(on))
}

/// Exposure: `false` = manual, `true` = auto.
pub fn camera_exposure_auto(cam: CameraId, auto: bool) -> Command {
    write(address::camera(cam.mid(), 0x28), on_off(auto))
}

/// Camera tally follow (HDMI 1–8 / SDI 1–8 as `VideoSource` 0x00–0x0F).
pub fn camera_tally_channel(cam: CameraId, source: VideoSource) -> Command {
    write(address::camera(cam.mid(), 0x29), source.value())
}

/// Generic parameter read.
pub fn read(address: Address) -> Command {
    Command::ReadParameter { address, size: 1 }
}

/// Generic parameter write.
pub fn write_parameter(address: Address, value: u8) -> Command {
    write(address, value)
}
