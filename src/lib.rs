//! Rust library for Roland video switcher remote control
//!
//! High-level Telnet API for Roland VR-6HD, V-160HD, and V-60HD (std environment).
//! Enable the `tokio` feature for [`AsyncTelnetClient`] and [`AsyncV60HdClient`].

pub use roland_core::*;

mod sync_client;
pub use sync_client::TelnetClient;

mod v60hd_client;
pub use v60hd_client::V60HdClient;

#[cfg(feature = "tokio")]
mod async_client;
#[cfg(feature = "tokio")]
pub use async_client::AsyncTelnetClient;

#[cfg(feature = "tokio")]
mod v60hd_async_client;
#[cfg(feature = "tokio")]
pub use v60hd_async_client::AsyncV60HdClient;

/// Error type for Telnet client
#[derive(Debug)]
pub enum TelnetError {
    /// Protocol-level error from roland-core
    Protocol(RolandError),
    /// I/O error
    Io(std::io::Error),
    /// Connection closed
    ConnectionClosed,
    /// V-160HD password prompt failed or welcome message was not received
    AuthenticationFailed,
}

impl std::fmt::Display for TelnetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelnetError::Protocol(e) => write!(f, "Protocol error: {}", e),
            TelnetError::Io(e) => write!(f, "I/O error: {}", e),
            TelnetError::ConnectionClosed => write!(f, "Connection closed"),
            TelnetError::AuthenticationFailed => write!(f, "Authentication failed"),
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

pub(crate) fn is_complete_telnet_response(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.ends_with(';')
        || trimmed.contains('\x06')
        || trimmed.contains('\x11')
        || trimmed.contains('\x13')
        || trimmed.eq_ignore_ascii_case("ack")
}
