use crate::devices::v60hd::{self, take_frame, Command, Response};
use crate::{RolandError, TelnetError};
use std::collections::VecDeque;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const IO_TIMEOUT: Duration = Duration::from_secs(5);

/// Asynchronous LAN client for Roland V-60HD (TCP 8023, STX framing, no password).
pub struct AsyncV60HdClient {
    stream: TcpStream,
    buffer: Vec<u8>,
    pending: VecDeque<Response>,
}

impl AsyncV60HdClient {
    /// Connect to a V-60HD on the default LAN CONTROL port (8023).
    pub async fn connect(host: &str) -> Result<Self, TelnetError> {
        Self::connect_on_port(host, v60hd::TELNET_PORT).await
    }

    /// Connect to a V-60HD on a custom TCP port.
    pub async fn connect_on_port(host: &str, port: u16) -> Result<Self, TelnetError> {
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
        })
    }

    /// Send one command and wait for ACK or a `;`-terminated query payload.
    pub async fn send(&mut self, command: &Command) -> Result<Response, TelnetError> {
        timeout(IO_TIMEOUT, async {
            self.stream.write_all(&command.encode_bytes()).await?;
            self.stream.flush().await
        })
        .await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "write timed out"))??;

        if command.is_query() {
            loop {
                match self.read_parsed_frame().await? {
                    Response::Acknowledge => continue,
                    Response::Error(e) => return Err(TelnetError::Protocol(e)),
                    other => {
                        self.drain_trailing_ack().await;
                        return Ok(other);
                    }
                }
            }
        } else {
            loop {
                match self.read_parsed_frame().await? {
                    Response::Acknowledge => {
                        self.drain_trailing_ack().await;
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
    pub async fn recv(&mut self) -> Result<Response, TelnetError> {
        loop {
            if let Some(pending) = self.pending.pop_front() {
                if matches!(pending, Response::Acknowledge) {
                    continue;
                }
                return Ok(pending);
            }
            match self.read_parsed_frame().await? {
                Response::Acknowledge => continue,
                other => return Ok(other),
            }
        }
    }

    /// `VER;` → product name and version string.
    pub async fn ver(&mut self) -> Result<(String, String), TelnetError> {
        match self.send(&v60hd::ver()).await? {
            Response::Version { product, version } => Ok((product, version)),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    /// `TLY;` → eight tally colors (channels 1–8).
    pub async fn tly(&mut self) -> Result<[v60hd::TallyColor; 8], TelnetError> {
        match self.send(&v60hd::tly()).await? {
            Response::Tally(colors) => Ok(colors),
            Response::Error(e) => Err(TelnetError::Protocol(e)),
            _ => Err(TelnetError::Protocol(RolandError::InvalidResponse)),
        }
    }

    /// `QPL:7;` → parsed panel snapshot.
    pub async fn qpl_all(&mut self) -> Result<v60hd::PanelStatus, TelnetError> {
        match self.send(&v60hd::qpl_all()).await? {
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

    async fn drain_trailing_ack(&mut self) {
        self.drain_buffered_ack();
        match timeout(Duration::from_millis(40), self.read_parsed_frame()).await {
            Ok(Ok(Response::Acknowledge)) => self.drain_buffered_ack(),
            Ok(Ok(other)) => self.pending.push_back(other),
            _ => {}
        }
    }

    async fn read_parsed_frame(&mut self) -> Result<Response, TelnetError> {
        loop {
            if let Some(frame) = take_frame(&mut self.buffer) {
                return v60hd::parse(&frame).map_err(TelnetError::from);
            }
            let mut buf = [0u8; 1024];
            let n = timeout(IO_TIMEOUT, self.stream.read(&mut buf))
                .await
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::TimedOut, "read timed out")
                })??;
            if n == 0 {
                return Err(TelnetError::ConnectionClosed);
            }
            self.buffer.extend_from_slice(&buf[..n]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn spawn_v60hd_stub() -> (u16, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 64];
            let n = socket.read(&mut buf).await.unwrap();
            assert_eq!(&buf[..n], v60hd::ver().encode_bytes());
            socket.write_all(b"\x02VER:V-60HD,3.10;").await.unwrap();

            let n = socket.read(&mut buf).await.unwrap();
            assert_eq!(&buf[..n], v60hd::cut().encode_bytes());
            socket.write_all(&[v60hd::ACK, v60hd::ACK]).await.unwrap();

            let n = socket.read(&mut buf).await.unwrap();
            assert_eq!(&buf[..n], v60hd::tly().encode_bytes());
            socket
                .write_all(b"\x11\x02TLY:1,2,0,0,0,0,0,0;\x06")
                .await
                .unwrap();
        });
        (port, handle)
    }

    #[tokio::test]
    async fn ver_without_ack_then_cut_ack_and_tly() {
        let (port, server) = spawn_v60hd_stub().await;
        let mut client = AsyncV60HdClient::connect_on_port("127.0.0.1", port)
            .await
            .unwrap();

        let (product, version) = client.ver().await.unwrap();
        assert_eq!(product, "V-60HD");
        assert_eq!(version, "3.10");

        let response = client.send(&v60hd::cut()).await.unwrap();
        assert!(matches!(response, Response::Acknowledge));

        let tally = client.tly().await.unwrap();
        assert_eq!(tally[0], v60hd::TallyColor::Red);
        assert_eq!(tally[1], v60hd::TallyColor::Green);

        server.await.unwrap();
    }
}
