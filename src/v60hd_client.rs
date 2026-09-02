use crate::devices::v60hd::{self, take_frame, Command, Response};
use crate::{RolandError, TelnetError};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Synchronous LAN client for Roland V-60HD (TCP 8023, STX framing, no password).
///
/// Only one TCP connection is accepted by the device at a time. Do not open
/// V-60HD RCS or another Telnet session while this client is connected.
pub struct V60HdClient {
    stream: TcpStream,
    buffer: Vec<u8>,
    pending: VecDeque<Response>,
}

impl V60HdClient {
    /// Connect to a V-60HD on the default LAN CONTROL port (8023).
    pub fn connect(host: &str) -> Result<Self, TelnetError> {
        Self::connect_on_port(host, v60hd::TELNET_PORT)
    }

    /// Connect to a V-60HD on a custom TCP port.
    pub fn connect_on_port(host: &str, port: u16) -> Result<Self, TelnetError> {
        let addr = format!("{host}:{port}");
        let stream = TcpStream::connect(&addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self {
            stream,
            buffer: Vec::new(),
            pending: VecDeque::new(),
        })
    }

    /// Send one command and wait for ACK or a `;`-terminated query payload.
    ///
    /// The next command is not written until this handshake completes.
    /// Unsolicited `QPL` / `TLY` frames are queued and available via [`recv`].
    pub fn send(&mut self, command: &Command) -> Result<Response, TelnetError> {
        self.stream.write_all(&command.encode_bytes())?;
        self.stream.flush()?;
        if command.is_query() {
            loop {
                match self.read_parsed_frame()? {
                    Response::Acknowledge => continue,
                    Response::Error(e) => return Err(TelnetError::Protocol(e)),
                    other => {
                        self.drain_trailing_ack();
                        return Ok(other);
                    }
                }
            }
        } else {
            loop {
                match self.read_parsed_frame()? {
                    Response::Acknowledge => {
                        self.drain_trailing_ack();
                        return Ok(Response::Acknowledge);
                    }
                    Response::Error(e) => return Err(TelnetError::Protocol(e)),
                    other => self.pending.push_back(other),
                }
            }
        }
    }

    /// Take a previously queued unsolicited response, or wait for the next frame.
    /// Duplicate ACKs from the device are skipped.
    pub fn recv(&mut self) -> Result<Response, TelnetError> {
        loop {
            if let Some(pending) = self.pending.pop_front() {
                if matches!(pending, Response::Acknowledge) {
                    continue;
                }
                return Ok(pending);
            }
            match self.read_parsed_frame()? {
                Response::Acknowledge => continue,
                other => return Ok(other),
            }
        }
    }

    /// `VER;` → product name and version string.
    pub fn ver(&mut self) -> Result<(String, String), TelnetError> {
        match self.send(&v60hd::ver())? {
            Response::Version { product, version } => Ok((product, version)),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    /// `TLY;` → eight tally colors (channels 1–8).
    pub fn tly(&mut self) -> Result<[v60hd::TallyColor; 8], TelnetError> {
        match self.send(&v60hd::tly())? {
            Response::Tally(colors) => Ok(colors),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    /// `QPL:7;` → parsed panel snapshot.
    pub fn qpl_all(&mut self) -> Result<v60hd::PanelStatus, TelnetError> {
        match self.send(&v60hd::qpl_all())? {
            Response::Panel { values } => {
                v60hd::PanelStatus::from_qpl_all(&values).map_err(TelnetError::from)
            }
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    fn drain_buffered_ack(&mut self) {
        if v60hd::next_is_ack(&self.buffer) {
            let _ = take_frame(&mut self.buffer);
        }
    }

    fn drain_trailing_ack(&mut self) {
        self.drain_buffered_ack();
        let _ = self
            .stream
            .set_read_timeout(Some(Duration::from_millis(40)));
        let mut buf = [0u8; 256];
        match self.stream.read(&mut buf) {
            Ok(0) => {}
            Ok(n) => {
                self.buffer.extend_from_slice(&buf[..n]);
                while let Some(frame) = take_frame(&mut self.buffer) {
                    match v60hd::parse(&frame) {
                        Ok(Response::Acknowledge) => {}
                        Ok(other) => self.pending.push_back(other),
                        Err(_) => break,
                    }
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::TimedOut
                    || e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => {}
        }
        let _ = self.stream.set_read_timeout(Some(Duration::from_secs(5)));
        self.drain_buffered_ack();
    }

    fn read_parsed_frame(&mut self) -> Result<Response, TelnetError> {
        loop {
            if let Some(frame) = take_frame(&mut self.buffer) {
                return v60hd::parse(&frame).map_err(TelnetError::from);
            }
            let mut buf = [0u8; 1024];
            let n = self.stream.read(&mut buf)?;
            if n == 0 {
                return Err(TelnetError::ConnectionClosed);
            }
            self.buffer.extend_from_slice(&buf[..n]);
        }
    }
}
