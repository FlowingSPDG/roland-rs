use crate::devices::v160hd;
use crate::{is_complete_telnet_response, Address, Command, Response, RolandError, TelnetError};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Telnet client for Roland video switchers
pub struct TelnetClient {
    stream: TcpStream,
    buffer: Vec<u8>,
    append_newline: bool,
}

impl TelnetClient {
    /// Connect to a device via Telnet (VR-6HD default port is 23).
    pub fn connect(host: &str, port: u16) -> Result<Self, TelnetError> {
        Self::connect_internal(host, port, false)
    }

    /// Connect to a V-160HD (TCP 8023) and send the 4-digit LAN password.
    pub fn connect_v160hd(host: &str, password: &str) -> Result<Self, TelnetError> {
        Self::connect_v160hd_on_port(host, v160hd::TELNET_PORT, password)
    }

    /// Connect to a V-160HD on a custom port.
    pub fn connect_v160hd_on_port(
        host: &str,
        port: u16,
        password: &str,
    ) -> Result<Self, TelnetError> {
        let mut client = Self::connect_internal(host, port, true)?;
        client.authenticate_v160hd(password)?;
        Ok(client)
    }

    fn connect_internal(host: &str, port: u16, append_newline: bool) -> Result<Self, TelnetError> {
        let addr = format!("{host}:{port}");
        let stream = TcpStream::connect(&addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self {
            stream,
            buffer: Vec::new(),
            append_newline,
        })
    }

    fn authenticate_v160hd(&mut self, password: &str) -> Result<(), TelnetError> {
        let prompt = self.read_until_contains("Enter password:")?;
        if !prompt.contains("Enter password:") {
            return Err(TelnetError::AuthenticationFailed);
        }
        self.stream.write_all(password.as_bytes())?;
        self.stream.write_all(b"\n")?;
        self.stream.flush()?;
        let welcome = self.read_until_contains("Welcome to V-160HD.")?;
        if !welcome.contains("Welcome to V-160HD.") {
            return Err(TelnetError::AuthenticationFailed);
        }
        self.buffer.clear();
        Ok(())
    }

    fn read_until_contains(&mut self, needle: &str) -> Result<String, TelnetError> {
        let mut buf = [0u8; 1024];
        loop {
            let n = self.stream.read(&mut buf)?;
            if n == 0 {
                return Err(TelnetError::ConnectionClosed);
            }
            self.buffer.extend_from_slice(&buf[..n]);
            let text = String::from_utf8_lossy(&self.buffer);
            if text.contains(needle) {
                return Ok(text.into_owned());
            }
        }
    }

    /// Send a command and wait for a response.
    pub fn send_command(&mut self, command: &Command) -> Result<Response, TelnetError> {
        let cmd_str = if self.append_newline {
            command.encode_line()
        } else {
            command.encode()
        };
        self.stream.write_all(cmd_str.as_bytes())?;
        self.stream.flush()?;
        self.read_response()
    }

    /// Send several write commands in sequence (e.g. 14-bit PinP parameters).
    pub fn send_commands(&mut self, commands: &[Command]) -> Result<(), TelnetError> {
        for command in commands {
            match self.send_command(command)? {
                Response::Acknowledge => {}
                Response::Error(e) => return Err(TelnetError::Protocol(e)),
                _ => return Err(TelnetError::Protocol(RolandError::InvalidResponse)),
            }
        }
        Ok(())
    }

    /// Press then release a V-160HD panel switch.
    pub fn press_and_release(&mut self, sw: Address) -> Result<(), TelnetError> {
        self.send_write(&v160hd::press_switch(sw))?;
        std::thread::sleep(Duration::from_millis(200));
        self.send_write(&v160hd::release_switch(sw))?;
        Ok(())
    }

    fn send_write(&mut self, command: &Command) -> Result<(), TelnetError> {
        match self.send_command(command)? {
            Response::Acknowledge => Ok(()),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    fn read_response(&mut self) -> Result<Response, TelnetError> {
        let mut buf = [0u8; 1024];
        let n = self.stream.read(&mut buf)?;

        if n == 0 {
            return Err(TelnetError::ConnectionClosed);
        }

        self.buffer.extend_from_slice(&buf[..n]);
        let response_str = String::from_utf8_lossy(&self.buffer);

        if is_complete_telnet_response(&response_str) {
            let response = Response::parse(&response_str)?;
            self.buffer.clear();
            Ok(response)
        } else {
            std::thread::sleep(Duration::from_millis(100));
            self.read_response()
        }
    }

    /// Write a parameter value.
    pub fn write_parameter(&mut self, address: &str, value: u8) -> Result<(), TelnetError> {
        let addr = Address::from_hex(address)?;
        let cmd = Command::WriteParameter {
            address: addr,
            value,
        };
        self.send_write(&cmd)
    }

    /// Read a parameter value.
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

    /// Get version information.
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
