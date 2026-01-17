//! Rust library for Roland VR-6HD remote control
//!
//! This library provides a high-level API for communicating with
//! Roland VR-6HD devices via Telnet (std environment).

pub use roland_core::*;

use roland_core::{Address, Command, Response, RolandError};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Error type for Telnet client
#[derive(Debug)]
pub enum TelnetError {
    /// Protocol-level error from roland-core
    Protocol(RolandError),
    /// I/O error
    Io(std::io::Error),
    /// Connection closed
    ConnectionClosed,
}

impl std::fmt::Display for TelnetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelnetError::Protocol(e) => write!(f, "Protocol error: {}", e),
            TelnetError::Io(e) => write!(f, "I/O error: {}", e),
            TelnetError::ConnectionClosed => write!(f, "Connection closed"),
        }
    }
}

impl std::error::Error for TelnetError {}

impl From<RolandError> for TelnetError {
    fn from(e: RolandError) -> Self {
        TelnetError::Protocol(e)
    }
}

impl From<std::io::Error> for TelnetError {
    fn from(e: std::io::Error) -> Self {
        TelnetError::Io(e)
    }
}

/// Telnet client for Roland VR-6HD
pub struct TelnetClient {
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl TelnetClient {
    /// Connect to VR-6HD device via Telnet
    ///
    /// # Arguments
    /// * `host` - IP address or hostname of the VR-6HD device
    /// * `port` - Telnet port (default: 23)
    ///
    /// # Returns
    /// * `Result<Self, TelnetError>` - Connected client or error
    pub fn connect(host: &str, port: u16) -> Result<Self, TelnetError> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)?;

        // Set read timeout
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        // Set write timeout
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        Ok(Self {
            stream,
            buffer: Vec::new(),
        })
    }

    /// Send a command and wait for response
    ///
    /// # Arguments
    /// * `command` - Command to send
    ///
    /// # Returns
    /// * `Result<Response, TelnetError>` - Response from device or error
    pub fn send_command(&mut self, command: &Command) -> Result<Response, TelnetError> {
        // Encode command (without STX for Telnet)
        let cmd_str = command.encode();
        let cmd_bytes = cmd_str.as_bytes();

        // Send command
        self.stream.write_all(cmd_bytes)?;
        self.stream.flush()?;

        // Read response
        self.read_response()
    }

    /// Read response from device
    fn read_response(&mut self) -> Result<Response, TelnetError> {
        let mut buf = [0u8; 1024];

        // Read data
        let n = self.stream.read(&mut buf)?;

        if n == 0 {
            return Err(TelnetError::ConnectionClosed);
        }

        // Append to buffer
        self.buffer.extend_from_slice(&buf[..n]);

        // Try to parse response
        // Responses typically end with ';' or control characters
        let response_str = String::from_utf8_lossy(&self.buffer);

        // Look for complete response (ends with ';' or is a control character)
        if response_str.ends_with(';') ||
           response_str.contains('\x06') || // ACK
           response_str.contains('\x11') || // XON
           response_str.contains('\x13')
        {
            // XOFF
            let response = Response::parse(&response_str)?;
            self.buffer.clear();
            Ok(response)
        } else {
            // Incomplete response, wait a bit and try again
            std::thread::sleep(Duration::from_millis(100));
            self.read_response()
        }
    }

    /// Write a parameter value
    ///
    /// # Arguments
    /// * `address` - SysEx address (3 bytes as hex string, e.g., "123456")
    /// * `value` - Value to write (0-255)
    ///
    /// # Returns
    /// * `Result<(), TelnetError>` - Success or error
    pub fn write_parameter(&mut self, address: &str, value: u8) -> Result<(), TelnetError> {
        let addr = Address::from_hex(address)?;
        let cmd = Command::WriteParameter {
            address: addr,
            value,
        };
        let response = self.send_command(&cmd)?;

        match response {
            Response::Acknowledge => Ok(()),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    /// Read a parameter value
    ///
    /// # Arguments
    /// * `address` - SysEx address (3 bytes as hex string, e.g., "123456")
    /// * `size` - Size to read (typically 1 for single byte)
    ///
    /// # Returns
    /// * `Result<u8, TelnetError>` - Parameter value or error
    pub fn read_parameter(&mut self, address: &str, size: u32) -> Result<u8, TelnetError> {
        let addr = Address::from_hex(address)?;
        let cmd = Command::ReadParameter {
            address: addr,
            size,
        };
        let response = self.send_command(&cmd)?;

        match response {
            Response::Data { value, .. } => Ok(value),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    /// Get version information
    ///
    /// # Returns
    /// * `Result<(String, String), TelnetError>` - (product, version) or error
    pub fn get_version(&mut self) -> Result<(String, String), TelnetError> {
        let cmd = Command::GetVersion;
        let response = self.send_command(&cmd)?;

        match response {
            Response::Version { product, version } => Ok((product, version)),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }
}
