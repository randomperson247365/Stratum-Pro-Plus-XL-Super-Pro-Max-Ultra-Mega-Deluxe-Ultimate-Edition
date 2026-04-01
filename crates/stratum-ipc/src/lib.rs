pub mod messages;

pub use messages::{IpcMessage, WindowInfo};

use std::env;
use std::path::PathBuf;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Returns the socket path: $XDG_RUNTIME_DIR/stratum.sock
pub fn socket_path() -> PathBuf {
    let dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(dir).join("stratum.sock")
}

/// The IPC server: binds the socket, accepts connections, and broadcasts messages
/// to all connected clients.
pub struct IpcServer {
    listener: UnixListener,
    pub tx: broadcast::Sender<IpcMessage>,
}

impl IpcServer {
    /// Bind the Unix socket and create the server.
    pub fn bind() -> Result<Self, IpcError> {
        let path = socket_path();
        // Remove stale socket if it exists.
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        let (tx, _) = broadcast::channel(256);
        Ok(Self { listener, tx })
    }

    /// Run the server loop: accept connections and route messages.
    /// Each connection gets a clone of the broadcast receiver; messages sent by
    /// any client are forwarded to all others.
    pub async fn run(self) {
        let tx = self.tx.clone();
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let tx_clone = tx.clone();
                    let rx = tx.subscribe();
                    tokio::spawn(handle_connection(stream, tx_clone, rx));
                }
                Err(e) => {
                    eprintln!("stratum-ipc: accept error: {e}");
                }
            }
        }
    }

    /// Send a message to all connected clients.
    pub fn broadcast(&self, msg: IpcMessage) {
        let _ = self.tx.send(msg);
    }
}

async fn handle_connection(
    stream: UnixStream,
    tx: broadcast::Sender<IpcMessage>,
    mut rx: broadcast::Receiver<IpcMessage>,
) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    loop {
        tokio::select! {
            // Read a message from this client and re-broadcast it.
            line = lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        match serde_json::from_str::<IpcMessage>(&text) {
                            Ok(msg) => { let _ = tx.send(msg); }
                            Err(e) => eprintln!("stratum-ipc: parse error: {e}"),
                        }
                    }
                    _ => break, // client disconnected
                }
            }
            // Forward broadcasts to this client.
            msg = rx.recv() => {
                match msg {
                    Ok(msg) => {
                        if let Ok(mut text) = serde_json::to_string(&msg) {
                            text.push('\n');
                            if writer.write_all(text.as_bytes()).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        }
    }
}

/// A simple IPC client for other components to connect and send/receive messages.
pub struct IpcClient {
    stream: BufReader<UnixStream>,
}

impl IpcClient {
    pub async fn connect() -> Result<Self, IpcError> {
        let path = socket_path();
        let stream = UnixStream::connect(&path).await?;
        Ok(Self { stream: BufReader::new(stream) })
    }

    pub async fn send(&mut self, msg: &IpcMessage) -> Result<(), IpcError> {
        let mut text = serde_json::to_string(msg)?;
        text.push('\n');
        self.stream.get_mut().write_all(text.as_bytes()).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<IpcMessage, IpcError> {
        let mut line = String::new();
        self.stream.read_line(&mut line).await?;
        Ok(serde_json::from_str(&line)?)
    }
}
