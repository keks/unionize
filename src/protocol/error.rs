extern crate alloc;
extern crate std;
use alloc::format;

use super::{DecodeError, EncodeError, ProtocolMonoid};

#[derive(Debug, Clone)]
pub enum RespondError<M: ProtocolMonoid> {
    EncodeError(M::EncodeError),
    DecodeError(M::DecodeError),
}

impl<M: ProtocolMonoid> From<EncodeError<M::EncodeError>> for RespondError<M> {
    fn from(value: EncodeError<M::EncodeError>) -> Self {
        Self::EncodeError(value.0)
    }
}

impl<M: ProtocolMonoid> From<DecodeError<M::DecodeError>> for RespondError<M> {
    fn from(value: DecodeError<M::DecodeError>) -> Self {
        Self::DecodeError(value.0)
    }
}

impl<M: ProtocolMonoid> std::error::Error for RespondError<M> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RespondError::EncodeError(e) => Some(e),
            RespondError::DecodeError(e) => Some(e),
        }
    }
}

impl<M: ProtocolMonoid> core::fmt::Display for RespondError<M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RespondError::EncodeError(e) => f.write_str(&format!("encoding error: {e}")),
            RespondError::DecodeError(e) => f.write_str(&format!("encoding error: {e}")),
        }
    }
}
