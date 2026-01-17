//! Core library for Roland VR-6HD remote control protocol
//!
//! This library provides the core functionality for communicating with
//! Roland VR-6HD devices via LAN/RS-232 interface.
//!
//! # Features
//!
//! - `no_std` compatible (requires `alloc` for string operations)
//! - Zero external dependencies
//! - Pure protocol implementation

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// Error types for Roland VR-6HD communication
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RolandError {
    /// Syntax error in received command
    SyntaxError,
    /// Invalid command due to other settings
    Invalid,
    /// Parameter out of range
    OutOfRange,
    /// Missing STX at command start (RS-232 only)
    NoStx,
    /// Unknown error code
    UnknownError(u8),
    /// Invalid address format
    InvalidAddress,
    /// Invalid value format
    InvalidValue,
    /// Invalid response format
    InvalidResponse,
}

impl fmt::Display for RolandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RolandError::SyntaxError => write!(f, "Syntax error in received command"),
            RolandError::Invalid => write!(f, "Invalid command due to other settings"),
            RolandError::OutOfRange => write!(f, "Parameter out of range"),
            RolandError::NoStx => write!(f, "Missing STX at command start"),
            RolandError::UnknownError(code) => write!(f, "Unknown error code: {}", code),
            RolandError::InvalidAddress => write!(f, "Invalid address format"),
            RolandError::InvalidValue => write!(f, "Invalid value format"),
            RolandError::InvalidResponse => write!(f, "Invalid response format"),
        }
    }
}

/// SysEx address (3 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address {
    /// High byte
    pub high: u8,
    /// Mid byte
    pub mid: u8,
    /// Low byte
    pub low: u8,
}

impl Address {
    /// Create a new address from three bytes
    pub fn new(high: u8, mid: u8, low: u8) -> Self {
        Self { high, mid, low }
    }

    /// Create an address from a hex string (6 hex digits)
    ///
    /// # Example
    /// ```
    /// use roland_core::Address;
    /// let addr = Address::from_hex("123456").unwrap();
    /// assert_eq!(addr.high, 0x12);
    /// assert_eq!(addr.mid, 0x34);
    /// assert_eq!(addr.low, 0x56);
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, RolandError> {
        if hex.len() != 6 {
            return Err(RolandError::InvalidAddress);
        }
        
        // Manual hex parsing to avoid std::str dependencies
        let high = parse_hex_byte(&hex[0..2])?;
        let mid = parse_hex_byte(&hex[2..4])?;
        let low = parse_hex_byte(&hex[4..6])?;
        
        Ok(Self { high, mid, low })
    }

    /// Convert address to hex string (6 hex digits, uppercase)
    ///
    /// Requires `alloc` for String allocation.
    pub fn to_hex(&self) -> String {
        format!("{:02X}{:02X}{:02X}", self.high, self.mid, self.low)
    }

    /// Write address as hex to a formatter
    ///
    /// This method doesn't require `alloc` and can be used in `no_std` environments
    /// without heap allocation.
    pub fn write_hex<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write_hex_byte(w, self.high)?;
        write_hex_byte(w, self.mid)?;
        write_hex_byte(w, self.low)
    }
}

/// Parse a single hex byte (2 hex digits)
fn parse_hex_byte(s: &str) -> Result<u8, RolandError> {
    if s.len() != 2 {
        return Err(RolandError::InvalidAddress);
    }
    
    let mut result = 0u8;
    for ch in s.chars() {
        let digit = match ch {
            '0'..='9' => ch as u8 - b'0',
            'A'..='F' => ch as u8 - b'A' + 10,
            'a'..='f' => ch as u8 - b'a' + 10,
            _ => return Err(RolandError::InvalidAddress),
        };
        result = result * 16 + digit;
    }
    Ok(result)
}

/// Write a byte as hex (2 hex digits, uppercase)
fn write_hex_byte<W: fmt::Write>(w: &mut W, byte: u8) -> fmt::Result {
    let high = (byte >> 4) & 0x0F;
    let low = byte & 0x0F;
    
    let high_char = if high < 10 {
        (b'0' + high) as char
    } else {
        (b'A' + high - 10) as char
    };
    
    let low_char = if low < 10 {
        (b'0' + low) as char
    } else {
        (b'A' + low - 10) as char
    };
    
    w.write_char(high_char)?;
    w.write_char(low_char)
}

/// Command types for VR-6HD
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Write parameter (DTH)
    WriteParameter {
        /// SysEx address
        address: Address,
        /// Value to write (0-255)
        value: u8,
    },
    /// Read parameter (RQH)
    ReadParameter {
        /// SysEx address
        address: Address,
        /// Size to read (typically 1 for single byte)
        size: u32,
    },
    /// Get version information (VER)
    GetVersion,
}

impl Command {
    /// Encode command to string format
    ///
    /// For Telnet, STX (0x02) is optional and omitted here.
    /// For RS-232, STX should be prepended by the transport layer.
    ///
    /// Requires `alloc` for String allocation.
    pub fn encode(&self) -> String {
        match self {
            Command::WriteParameter { address, value } => {
                format!("DTH:{},{:02X};", address.to_hex(), value)
            }
            Command::ReadParameter { address, size } => {
                // Size is 3 bytes in hex (6 hex digits)
                let size_hex = format!("{:06X}", size);
                format!("RQH:{},{};", address.to_hex(), size_hex)
            }
            Command::GetVersion => "VER;".to_string(),
        }
    }

    /// Encode command with STX prefix (for RS-232)
    ///
    /// Requires `alloc` for String allocation.
    pub fn encode_with_stx(&self) -> String {
        format!("\x02{}", self.encode())
    }

    /// Write command to a formatter
    ///
    /// This method doesn't require `alloc` and can be used in `no_std` environments
    /// without heap allocation.
    pub fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            Command::WriteParameter { address, value } => {
                w.write_str("DTH:")?;
                address.write_hex(w)?;
                w.write_str(",")?;
                write_hex_byte(w, *value)?;
                w.write_str(";")
            }
            Command::ReadParameter { address, size } => {
                w.write_str("RQH:")?;
                address.write_hex(w)?;
                w.write_str(",")?;
                // Size is 3 bytes in hex (6 hex digits)
                write_hex_u24(w, *size)?;
                w.write_str(";")
            }
            Command::GetVersion => w.write_str("VER;"),
        }
    }

    /// Write command with STX prefix to a formatter
    pub fn write_with_stx<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_char('\x02')?;
        self.write(w)
    }
}

/// Write a 24-bit value as hex (6 hex digits, uppercase)
fn write_hex_u24<W: fmt::Write>(w: &mut W, value: u32) -> fmt::Result {
    write_hex_byte(w, ((value >> 16) & 0xFF) as u8)?;
    write_hex_byte(w, ((value >> 8) & 0xFF) as u8)?;
    write_hex_byte(w, (value & 0xFF) as u8)
}

/// Response types from VR-6HD
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// Acknowledge (ack)
    Acknowledge,
    /// Data response (DTH)
    Data {
        /// SysEx address
        address: Address,
        /// Parameter value
        value: u8,
    },
    /// Version information (VER)
    Version {
        /// Product name
        product: String,
        /// Version string
        version: String,
    },
    /// Error response (ERR)
    Error(RolandError),
}

impl Response {
    /// Parse response from string slice
    ///
    /// Handles both Telnet (no STX) and RS-232 (with STX) formats.
    ///
    /// Requires `alloc` for String allocation in Version response.
    pub fn parse(response: &str) -> Result<Self, RolandError> {
        let response = response.trim();
        
        // Remove STX if present (0x02)
        let response = if response.starts_with('\x02') {
            &response[1..]
        } else {
            response
        };

        // Handle ACK (0x06)
        if response == "\x06" || response == "ack" {
            return Ok(Response::Acknowledge);
        }

        // Handle XON/XOFF (flow control)
        // These are handled by the transport layer, but we can detect them
        if response == "\x11" || response == "xon" {
            // XON - can continue sending (not an error, but transport layer should handle)
            return Err(RolandError::InvalidResponse);
        }
        if response == "\x13" || response == "xoff" {
            // XOFF - pause sending (not an error, but transport layer should handle)
            return Err(RolandError::InvalidResponse);
        }

        // Parse DTH response: DTH:address,value;
        if response.starts_with("DTH:") {
            let content = &response[4..];
            if !content.ends_with(';') {
                return Err(RolandError::InvalidResponse);
            }
            let content = &content[..content.len() - 1];
            let parts: Vec<&str> = content.split(',').collect();
            if parts.len() != 2 {
                return Err(RolandError::InvalidResponse);
            }
            let address = Address::from_hex(parts[0])?;
            let value = parse_hex_byte(parts[1])?;
            return Ok(Response::Data { address, value });
        }

        // Parse VER response: VER:product,version;
        if response.starts_with("VER:") {
            let content = &response[4..];
            if !content.ends_with(';') {
                return Err(RolandError::InvalidResponse);
            }
            let content = &content[..content.len() - 1];
            let parts: Vec<&str> = content.split(',').collect();
            if parts.len() != 2 {
                return Err(RolandError::InvalidResponse);
            }
            return Ok(Response::Version {
                product: parts[0].to_string(),
                version: parts[1].to_string(),
            });
        }

        // Parse ERR response: ERR:code;
        if response.starts_with("ERR:") {
            let content = &response[4..];
            if !content.ends_with(';') {
                return Err(RolandError::InvalidResponse);
            }
            let content = &content[..content.len() - 1];
            let code = parse_decimal_u8(content)?;
            let error = match code {
                0 => RolandError::SyntaxError,
                4 => RolandError::Invalid,
                5 => RolandError::OutOfRange,
                6 => RolandError::NoStx,
                _ => RolandError::UnknownError(code),
            };
            return Ok(Response::Error(error));
        }

        Err(RolandError::InvalidResponse)
    }
}

/// Parse a decimal u8
fn parse_decimal_u8(s: &str) -> Result<u8, RolandError> {
    let mut result = 0u8;
    for ch in s.chars() {
        let digit = match ch {
            '0'..='9' => ch as u8 - b'0',
            _ => return Err(RolandError::InvalidResponse),
        };
        result = result.checked_mul(10)
            .and_then(|r| r.checked_add(digit))
            .ok_or(RolandError::InvalidResponse)?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_from_hex() {
        let addr = Address::from_hex("123456").unwrap();
        assert_eq!(addr.high, 0x12);
        assert_eq!(addr.mid, 0x34);
        assert_eq!(addr.low, 0x56);
    }

    #[test]
    fn test_address_to_hex() {
        let addr = Address::new(0x12, 0x34, 0x56);
        assert_eq!(addr.to_hex(), "123456");
    }

    #[test]
    fn test_address_write_hex() {
        let addr = Address::new(0x12, 0x34, 0x56);
        let mut s = String::new();
        addr.write_hex(&mut s).unwrap();
        assert_eq!(s, "123456");
    }

    #[test]
    fn test_write_command() {
        let cmd = Command::WriteParameter {
            address: Address::from_hex("123456").unwrap(),
            value: 0x01,
        };
        assert_eq!(cmd.encode(), "DTH:123456,01;");
    }

    #[test]
    fn test_write_command_write() {
        let cmd = Command::WriteParameter {
            address: Address::from_hex("123456").unwrap(),
            value: 0x01,
        };
        let mut s = String::new();
        cmd.write(&mut s).unwrap();
        assert_eq!(s, "DTH:123456,01;");
    }

    #[test]
    fn test_read_command() {
        let cmd = Command::ReadParameter {
            address: Address::from_hex("123456").unwrap(),
            size: 1,
        };
        assert_eq!(cmd.encode(), "RQH:123456,000001;");
    }

    #[test]
    fn test_version_command() {
        let cmd = Command::GetVersion;
        assert_eq!(cmd.encode(), "VER;");
    }

    #[test]
    fn test_parse_ack() {
        let resp = Response::parse("\x06").unwrap();
        assert_eq!(resp, Response::Acknowledge);
    }

    #[test]
    fn test_parse_data() {
        let resp = Response::parse("DTH:123456,01;").unwrap();
        match resp {
            Response::Data { address, value } => {
                assert_eq!(address.to_hex(), "123456");
                assert_eq!(value, 0x01);
            }
            _ => panic!("Expected Data response"),
        }
    }

    #[test]
    fn test_parse_version() {
        let resp = Response::parse("VER:VR-6HD,1.00;").unwrap();
        match resp {
            Response::Version { product, version } => {
                assert_eq!(product, "VR-6HD");
                assert_eq!(version, "1.00");
            }
            _ => panic!("Expected Version response"),
        }
    }

    #[test]
    fn test_parse_error() {
        let resp = Response::parse("ERR:0;").unwrap();
        match resp {
            Response::Error(RolandError::SyntaxError) => {}
            _ => panic!("Expected SyntaxError"),
        }
    }
}
