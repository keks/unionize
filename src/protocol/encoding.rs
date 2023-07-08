extern crate alloc;
extern crate std;

use alloc::format;

#[derive(Debug, Clone)]
pub struct EncodeError<E>(pub E);
#[derive(Debug, Clone)]
pub struct DecodeError<E>(pub E);

pub trait Encodable: Default {
    type Encoded: Clone + core::fmt::Debug + Eq + Default;
    type EncodeError: std::error::Error + core::fmt::Debug + 'static;
    type DecodeError: std::error::Error + core::fmt::Debug + 'static;

    fn encode(&self, encoded: &mut Self::Encoded) -> Result<(), EncodeError<Self::EncodeError>>;
    fn decode(&mut self, encoded: &Self::Encoded) -> Result<(), DecodeError<Self::DecodeError>>;

    fn to_encoded(&self) -> Result<Self::Encoded, EncodeError<Self::EncodeError>> {
        let mut encoded = Self::Encoded::default();
        self.encode(&mut encoded)?;
        Ok(encoded)
    }

    fn from_encoded(encoded: &Self::Encoded) -> Result<Self, DecodeError<Self::DecodeError>> {
        let mut decoded = Self::default();
        decoded.decode(encoded)?;
        Ok(decoded)
    }

    fn batch_encode<Dst: AsDestMutRef<Self::Encoded>>(
        src: &[Self],
        dst: &mut [Dst],
    ) -> Result<(), EncodeError<Self::EncodeError>> {
        assert_eq!(
            src.len(),
            dst.len(),
            "source and destination count doesn't match"
        );
        for i in 0..src.len() {
            src[i].encode(dst[i].as_dest_mut_ref())?;
        }

        Ok(())
    }
}

pub trait AsDestMutRef<T> {
    fn as_dest_mut_ref(&mut self) -> &mut T;
}

// impl<T> AsDestMutRef<T> for T {
//     fn as_dest_mut_ref(&mut self) -> &mut T {
//         self
//     }
// }

impl<E: std::error::Error> core::fmt::Display for EncodeError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&format!("encoding error: {}", self.0))
    }
}

impl<D: std::error::Error> core::fmt::Display for DecodeError<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&format!("decoding error: {}", self.0))
    }
}

impl<E: std::error::Error + 'static> std::error::Error for EncodeError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl<D: std::error::Error + 'static> std::error::Error for DecodeError<D> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
