//! V-60HD response framing and parse.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::types::TallyColor;
use crate::RolandError;

/// STX required on every V-60HD command (LAN and RS-232).
pub const STX: u8 = 0x02;
/// ACK returned after an accepted command (except some queries such as `VER`).
pub const ACK: u8 = 0x06;
/// XON flow-control byte (skipped by the client).
pub const XON: u8 = 0x11;
/// XOFF flow-control byte (skipped by the client).
pub const XOFF: u8 = 0x13;

/// Parsed V-60HD reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// ACK (0x06 or ASCII `ACK`).
    Acknowledge,
    /// `ERR:a;`
    Error(RolandError),
    /// `TLY:a,...,h;` eight tally colors for channels 1–8.
    Tally([TallyColor; 8]),
    /// `VER:V-60HD,version;`
    Version { product: String, version: String },
    /// `QPL:...;` decimal fields (Companion uses index 0–6 for ALL).
    Panel { values: Vec<i32> },
    /// `QAL:...;` one or more levels in tenths of a dB (`-801` = -INF).
    AudioLevels { values: Vec<i32> },
    /// `ACS:...;` raw argument tokens until the firmware layout is confirmed.
    Status { args: Vec<String> },
}

/// True when the next non-noise byte is ACK (does not consume the buffer).
pub fn next_is_ack(buffer: &[u8]) -> bool {
    matches!(
        buffer
            .iter()
            .find(|&&b| !matches!(b, XON | XOFF | b'\r' | b'\n' | b' ' | b'\t')),
        Some(&ACK)
    )
}

/// Drain leading XON/XOFF/whitespace and return the next ACK or `;` frame.
pub fn take_frame(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    drain_noise(buffer);
    if buffer.is_empty() {
        return None;
    }
    if buffer[0] == ACK {
        return Some(buffer.drain(..1).collect());
    }
    let end = buffer.iter().position(|&b| b == b';')?;
    Some(buffer.drain(..=end).collect())
}

fn drain_noise(buffer: &mut Vec<u8>) {
    let n = buffer
        .iter()
        .position(|&b| !matches!(b, XON | XOFF | b'\r' | b'\n' | b' ' | b'\t'))
        .unwrap_or(buffer.len());
    buffer.drain(..n);
}

/// Parse a complete ACK byte or a `;`-terminated ASCII frame.
pub fn parse(bytes: &[u8]) -> Result<Response, RolandError> {
    let bytes = trim_ascii(bytes);
    if bytes.is_empty() {
        return Err(RolandError::InvalidResponse);
    }
    if bytes == [ACK] || eq_ignore_ascii_case(bytes, b"ack") {
        return Ok(Response::Acknowledge);
    }

    let bytes = match bytes.first().copied() {
        Some(STX) => &bytes[1..],
        _ => bytes,
    };
    let text = core::str::from_utf8(bytes).map_err(|_| RolandError::InvalidResponse)?;
    let text = text.trim();
    if !text.ends_with(';') {
        return Err(RolandError::InvalidResponse);
    }
    let text = &text[..text.len() - 1];
    if text.len() < 3 {
        return Err(RolandError::InvalidResponse);
    }
    let opcode = &text[..3];
    let args = if text.len() == 3 {
        ""
    } else if text.as_bytes()[3] == b':' {
        &text[4..]
    } else {
        return Err(RolandError::InvalidResponse);
    };

    match opcode {
        "ERR" => parse_err(args),
        "TLY" => parse_tly(args),
        "VER" => parse_ver(args),
        "QPL" => Ok(Response::Panel {
            values: parse_i32_list(args)?,
        }),
        "QAL" => Ok(Response::AudioLevels {
            values: parse_i32_list(args)?,
        }),
        "ACS" => Ok(Response::Status {
            args: split_args(args),
        }),
        _ => Err(RolandError::InvalidResponse),
    }
}

fn parse_err(args: &str) -> Result<Response, RolandError> {
    let code = parse_i32(args)?;
    let error = match code {
        0 => RolandError::SyntaxError,
        4 => RolandError::Invalid,
        5 => RolandError::OutOfRange,
        other => {
            if (0..=255).contains(&other) {
                RolandError::UnknownError(other as u8)
            } else {
                return Err(RolandError::InvalidResponse);
            }
        }
    };
    Ok(Response::Error(error))
}

fn parse_tly(args: &str) -> Result<Response, RolandError> {
    let values = parse_i32_list(args)?;
    if values.len() != 8 {
        return Err(RolandError::InvalidResponse);
    }
    let mut colors = [TallyColor::Dark; 8];
    for (slot, value) in colors.iter_mut().zip(values) {
        let byte = u8::try_from(value).map_err(|_| RolandError::InvalidResponse)?;
        *slot = TallyColor::from_u8(byte)?;
    }
    Ok(Response::Tally(colors))
}

fn parse_ver(args: &str) -> Result<Response, RolandError> {
    let mut parts = args.splitn(2, ',');
    let product = parts.next().ok_or(RolandError::InvalidResponse)?;
    let version = parts.next().ok_or(RolandError::InvalidResponse)?;
    if product.is_empty() || version.is_empty() {
        return Err(RolandError::InvalidResponse);
    }
    Ok(Response::Version {
        product: product.to_string(),
        version: version.to_string(),
    })
}

fn parse_i32_list(args: &str) -> Result<Vec<i32>, RolandError> {
    if args.is_empty() {
        return Ok(Vec::new());
    }
    args.split(',').map(parse_i32).collect()
}

fn split_args(args: &str) -> Vec<String> {
    if args.is_empty() {
        Vec::new()
    } else {
        args.split(',').map(ToString::to_string).collect()
    }
}

fn parse_i32(s: &str) -> Result<i32, RolandError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(RolandError::InvalidResponse);
    }
    let (sign, digits) = if let Some(rest) = s.strip_prefix('-') {
        (-1i32, rest)
    } else {
        (1i32, s)
    };
    if digits.is_empty() {
        return Err(RolandError::InvalidResponse);
    }
    let mut value = 0i32;
    for ch in digits.chars() {
        let digit = ch.to_digit(10).ok_or(RolandError::InvalidResponse)?;
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add(digit as i32))
            .ok_or(RolandError::InvalidResponse)?;
    }
    value.checked_mul(sign).ok_or(RolandError::InvalidResponse)
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|&b| !matches!(b, XON | XOFF | b'\r' | b'\n' | b' ' | b'\t'))
        .unwrap_or(bytes.len());
    let rest = &bytes[start..];
    let end = rest
        .iter()
        .rposition(|&b| !matches!(b, XON | XOFF | b'\r' | b'\n' | b' ' | b'\t'))
        .map(|i| i + 1)
        .unwrap_or(0);
    &rest[..end]
}

fn eq_ignore_ascii_case(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.len() == needle.len()
        && bytes
            .iter()
            .zip(needle)
            .all(|(a, b)| a.to_ascii_lowercase() == *b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn take_frame_ack_then_tly() {
        let mut buf = vec![ACK];
        buf.extend_from_slice(b"\x02TLY:1,2,0,0,0,0,0,0;");
        buf.push(ACK);
        assert_eq!(take_frame(&mut buf), Some(vec![ACK]));
        assert_eq!(
            take_frame(&mut buf),
            Some(b"\x02TLY:1,2,0,0,0,0,0,0;".to_vec())
        );
        assert_eq!(take_frame(&mut buf), Some(vec![ACK]));
        assert!(take_frame(&mut buf).is_none());
    }

    #[test]
    fn take_frame_skips_xon_xoff() {
        let mut buf = vec![XON, XOFF, ACK];
        assert_eq!(take_frame(&mut buf), Some(vec![ACK]));
    }

    #[test]
    fn take_frame_incomplete_returns_none() {
        let mut buf = b"\x02TLY:1,2".to_vec();
        assert!(take_frame(&mut buf).is_none());
        assert_eq!(buf, b"\x02TLY:1,2");
    }

    #[test]
    fn parse_ack_byte_and_ascii() {
        assert_eq!(parse(&[ACK]).unwrap(), Response::Acknowledge);
        assert_eq!(parse(b"ACK").unwrap(), Response::Acknowledge);
        assert_eq!(parse(b"ack").unwrap(), Response::Acknowledge);
    }

    #[test]
    fn parse_tly_official_example() {
        let resp = parse(b"\x02TLY:1,2,0,0,0,0,0,0;").unwrap();
        match resp {
            Response::Tally(colors) => {
                assert_eq!(colors[0], TallyColor::Red);
                assert_eq!(colors[1], TallyColor::Green);
                assert!(colors[2..].iter().all(|c| *c == TallyColor::Dark));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parse_ver() {
        let resp = parse(b"\x02VER:V-60HD,3.10;").unwrap();
        match resp {
            Response::Version { product, version } => {
                assert_eq!(product, "V-60HD");
                assert_eq!(version, "3.10");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parse_err_codes() {
        assert_eq!(
            parse(b"ERR:0;").unwrap(),
            Response::Error(RolandError::SyntaxError)
        );
        assert_eq!(
            parse(b"\x02ERR:4;").unwrap(),
            Response::Error(RolandError::Invalid)
        );
        assert_eq!(
            parse(b"ERR:5;").unwrap(),
            Response::Error(RolandError::OutOfRange)
        );
    }

    #[test]
    fn parse_qpl_and_qal() {
        match parse(b"QPL:0,1,2,0,1,0,10;").unwrap() {
            Response::Panel { values } => {
                assert_eq!(values, vec![0, 1, 2, 0, 1, 0, 10]);
            }
            other => panic!("unexpected {other:?}"),
        }
        match parse(b"QAL:-801,0,100;").unwrap() {
            Response::AudioLevels { values } => {
                assert_eq!(values, vec![-801, 0, 100]);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parse_acs_keeps_raw_args() {
        match parse(b"ACS:1,2,foo;").unwrap() {
            Response::Status { args } => {
                assert_eq!(
                    args,
                    vec!["1".to_string(), "2".to_string(), "foo".to_string()]
                );
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
