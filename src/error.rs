use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error("Discord RPC error: {0}")]
  Rpc(String),
  #[error("Not connected to Discord")]
  NotConnected,
  #[error("Already connected to Discord")]
  AlreadyConnected,
  #[error("Operation not supported on this platform")]
  Unsupported,
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}
