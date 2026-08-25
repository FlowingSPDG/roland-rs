use crate::devices::v160hd;
use crate::{Address, Command, Response, RolandError, TelnetError};
use std::collections::VecDeque;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const IO_TIMEOUT: Duration = Duration::from_secs(5);

/// Asynchronous Telnet client for Roland video switchers.
pub struct AsyncTelnetClient {
    stream: TcpStream,
    buffer: Vec<u8>,
    pending: VecDeque<Response>,
    append_newline: bool,
}

impl AsyncTelnetClient {
    /// Connect to a device via Telnet (VR-6HD default port is 23).
    pub async fn connect(host: &str, port: u16) -> Result<Self, TelnetError> {
        Self::connect_internal(host, port, false).await
    }

    /// Connect to a V-160HD (TCP 8023) and send the 4-digit LAN password.
    pub async fn connect_v160hd(host: &str, password: &str) -> Result<Self, TelnetError> {
        Self::connect_v160hd_on_port(host, v160hd::TELNET_PORT, password).await
    }

    /// Connect to a V-160HD on a custom port.
    pub async fn connect_v160hd_on_port(
        host: &str,
        port: u16,
        password: &str,
    ) -> Result<Self, TelnetError> {
        let mut client = Self::connect_internal(host, port, true).await?;
        client.authenticate_v160hd(password).await?;
        Ok(client)
    }

    async fn connect_internal(
        host: &str,
        port: u16,
        append_newline: bool,
    ) -> Result<Self, TelnetError> {
        let addr = format!("{host}:{port}");
        let stream = timeout(IO_TIMEOUT, TcpStream::connect(&addr))
            .await
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::TimedOut, "connect timed out")
            })??;
        Ok(Self {
            stream,
            buffer: Vec::new(),
            pending: VecDeque::new(),
            append_newline,
        })
    }

    async fn authenticate_v160hd(&mut self, password: &str) -> Result<(), TelnetError> {
        let prompt = self.read_until_contains("Enter password:").await?;
        if !prompt.contains("Enter password:") {
            return Err(TelnetError::AuthenticationFailed);
        }
        timeout(IO_TIMEOUT, async {
            self.stream.write_all(password.as_bytes()).await?;
            self.stream.write_all(b"\n").await?;
            self.stream.flush().await
        })
        .await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "write timed out"))??;
        let welcome = self.read_until_contains("Welcome to V-160HD.").await?;
        if !welcome.contains("Welcome to V-160HD.") {
            return Err(TelnetError::AuthenticationFailed);
        }
        self.buffer.clear();
        Ok(())
    }

    async fn read_until_contains(&mut self, needle: &str) -> Result<String, TelnetError> {
        let mut buf = [0u8; 1024];
        loop {
            let n = timeout(IO_TIMEOUT, self.stream.read(&mut buf))
                .await
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::TimedOut, "read timed out")
                })??;
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
    pub async fn send_command(&mut self, command: &Command) -> Result<Response, TelnetError> {
        let cmd_str = if self.append_newline {
            command.encode_line()
        } else {
            command.encode()
        };
        timeout(IO_TIMEOUT, async {
            self.stream.write_all(cmd_str.as_bytes()).await?;
            self.stream.flush().await
        })
        .await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "write timed out"))??;
        loop {
            let response = self.read_response().await?;
            if is_tally_notify(&response) && !command_expects_tally(command) {
                self.pending.push_back(response);
                continue;
            }
            return Ok(response);
        }
    }

    /// Wait for the next incoming response, including unsolicited tally DTH.
    pub async fn recv(&mut self) -> Result<Response, TelnetError> {
        if let Some(pending) = self.pending.pop_front() {
            return Ok(pending);
        }
        self.read_frame(None).await
    }

    /// Send several write commands in sequence (e.g. 14-bit PinP parameters).
    pub async fn send_commands(&mut self, commands: &[Command]) -> Result<(), TelnetError> {
        for command in commands {
            match self.send_command(command).await? {
                Response::Acknowledge => {}
                Response::Error(e) => return Err(TelnetError::Protocol(e)),
                _ => return Err(TelnetError::Protocol(RolandError::InvalidResponse)),
            }
        }
        Ok(())
    }

    /// Press then release a V-160HD panel switch.
    pub async fn press_and_release(&mut self, sw: Address) -> Result<(), TelnetError> {
        self.send_write(&v160hd::press_switch(sw)).await?;
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.send_write(&v160hd::release_switch(sw)).await?;
        Ok(())
    }

    async fn send_write(&mut self, command: &Command) -> Result<(), TelnetError> {
        match self.send_command(command).await? {
            Response::Acknowledge => Ok(()),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    async fn read_response(&mut self) -> Result<Response, TelnetError> {
        self.read_frame(Some(IO_TIMEOUT)).await
    }

    async fn read_frame(&mut self, io_timeout: Option<Duration>) -> Result<Response, TelnetError> {
        loop {
            if let Some(frame) = take_complete_frame(&mut self.buffer) {
                return Response::parse(&frame).map_err(TelnetError::from);
            }
            let mut buf = [0u8; 1024];
            let n = if let Some(limit) = io_timeout {
                timeout(limit, self.stream.read(&mut buf))
                    .await
                    .map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::TimedOut, "read timed out")
                    })??
            } else {
                self.stream.read(&mut buf).await?
            };
            if n == 0 {
                return Err(TelnetError::ConnectionClosed);
            }
            self.buffer.extend_from_slice(&buf[..n]);
        }
    }

    /// Write a parameter value.
    pub async fn write_parameter(&mut self, address: &str, value: u8) -> Result<(), TelnetError> {
        let addr = Address::from_hex(address)?;
        let cmd = Command::WriteParameter {
            address: addr,
            value,
        };
        self.send_write(&cmd).await
    }

    /// Read a parameter value.
    pub async fn read_parameter(&mut self, address: &str, size: u32) -> Result<u8, TelnetError> {
        let addr = Address::from_hex(address)?;
        let cmd = Command::ReadParameter {
            address: addr,
            size,
        };
        let response = self.send_command(&cmd).await?;

        match response {
            Response::Data { value, .. } => Ok(value),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    /// Get version information.
    pub async fn get_version(&mut self) -> Result<(String, String), TelnetError> {
        let cmd = Command::GetVersion;
        let response = self.send_command(&cmd).await?;

        match response {
            Response::Version { product, version } => Ok((product, version)),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }
}

fn take_complete_frame(buffer: &mut Vec<u8>) -> Option<String> {
    let s = std::str::from_utf8(buffer).ok()?;
    let start = s.find(|c: char| !c.is_whitespace())?;
    let rest = &s[start..];
    if rest.len() >= 3 && rest[..3].eq_ignore_ascii_case("ack") {
        let mut end = start + 3;
        while end < s.len() && matches!(s.as_bytes()[end], b'\r' | b'\n' | b' ' | b'\t') {
            end += 1;
            if buffer[end - 1] == b'\n' {
                break;
            }
        }
        buffer.drain(..end);
        return Some("ACK".to_string());
    }
    let rel = rest.find(';')?;
    let end = start + rel + 1;
    let frame = s[start..end].to_string();
    let mut drain_to = end;
    while drain_to < s.len() && matches!(s.as_bytes()[drain_to], b'\r' | b'\n') {
        drain_to += 1;
    }
    buffer.drain(..drain_to);
    Some(frame)
}

fn is_tally_notify(response: &Response) -> bool {
    match response {
        Response::Data { address, .. } | Response::DataBlock { address, .. } => {
            address.high == 0x0C && address.mid == 0x00
        }
        _ => false,
    }
}

fn command_expects_tally(command: &Command) -> bool {
    matches!(
        command,
        Command::ReadParameter { address, .. } if address.high == 0x0C && address.mid == 0x00
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::v160hd;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn spawn_v160hd_stub() -> (u16, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            socket.write_all(b"Enter password:\n").await.unwrap();
            let mut buf = [0u8; 32];
            let _ = socket.read(&mut buf).await.unwrap();
            socket.write_all(b"Welcome to V-160HD.\n").await.unwrap();
            let mut cmd = [0u8; 64];
            let n = socket.read(&mut cmd).await.unwrap();
            assert!(std::str::from_utf8(&cmd[..n]).unwrap().contains("DTH:"));
            socket.write_all(b"ACK\n").await.unwrap();
        });
        (port, handle)
    }

    #[tokio::test]
    async fn authenticates_and_sends_write() {
        let (port, server) = spawn_v160hd_stub().await;
        let mut client = AsyncTelnetClient::connect_v160hd_on_port("127.0.0.1", port, "0000")
            .await
            .unwrap();
        let hdmi1 = v160hd::VideoSource::hdmi(1).unwrap();
        let response = client
            .send_command(&v160hd::select_pgm(hdmi1))
            .await
            .unwrap();
        assert!(matches!(response, Response::Acknowledge));
        server.await.unwrap();
    }

    #[test]
    fn take_complete_frame_splits_ack_and_dth() {
        let mut buf = b"ACK\nDTH:0C0000,0001;\n".to_vec();
        assert_eq!(take_complete_frame(&mut buf).as_deref(), Some("ACK"));
        assert_eq!(
            take_complete_frame(&mut buf).as_deref(),
            Some("DTH:0C0000,0001;")
        );
        assert!(buf.is_empty() || buf.iter().all(|b| b.is_ascii_whitespace()));
    }
}
