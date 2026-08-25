//! MIDI-compatible value encoding used by Roland SysEx parameters.
//!
//! Multi-byte numeric parameters on V-160HD (and related models) are stored as
//! two 7-bit bytes. Negative values use 14-bit two's complement, then mask
//! `0x3FFF`. Companion's `calculateBytes()` implements the same rule.

/// Encode a scaled integer as two 7-bit MIDI bytes `(msb, lsb)`.
///
/// `scaled` is the already-multiplied integer (for example position `-100.0`
/// with scale `10` is `-1000`).
pub fn encode_14bit(scaled: i32) -> (u8, u8) {
    let masked = (scaled as u32) & 0x3FFF;
    let lsb = (masked & 0x7F) as u8;
    let msb = ((masked >> 7) & 0x7F) as u8;
    (msb, lsb)
}

/// Decode two 7-bit MIDI bytes into a 14-bit unsigned value.
pub fn decode_14bit(msb: u8, lsb: u8) -> u16 {
    (((msb as u16) & 0x7F) << 7) | ((lsb as u16) & 0x7F)
}

/// Encode a signed 7-bit value (`-64..=63`) as a single SysEx byte.
///
/// Used for V-160HD PinP hue width (`-30..=30` → `0x62..=0x1E`).
pub fn encode_s7(value: i8) -> u8 {
    (value as u8) & 0x7F
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_positive_scaled() {
        // 50.0 * 10 = 500 → msb=3, lsb=116
        assert_eq!(encode_14bit(500), (0x03, 0x74));
    }

    #[test]
    fn encode_negative_scaled() {
        // -100.0 * 10 = -1000 → 14-bit two's complement
        assert_eq!(encode_14bit(-1000), (0x78, 0x18));
    }

    #[test]
    fn encode_hue_width() {
        assert_eq!(encode_s7(-30), 0x62);
        assert_eq!(encode_s7(0), 0x00);
        assert_eq!(encode_s7(30), 0x1E);
    }

    #[test]
    fn roundtrip_14bit() {
        let (msb, lsb) = encode_14bit(255);
        assert_eq!(decode_14bit(msb, lsb), 255);
    }
}
