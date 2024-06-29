use kv_3d_storage::*;

/// A `u8` that uses a fixed-width homomorphic encoding.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct U8FixedWidth(pub u8);

impl Dimension for U8FixedWidth {
    const HOMOMORPHIC_ENCODING_MAX_LENGTH: usize = 1;

    const IS_FIXED_WIDTH_ENCODING: bool = true;

    fn homomorphic_encode(&self, buf: &mut [u8]) -> usize {
        buf[0] = self.0;
        return 1;
    }

    fn homomorphic_decode(buf: &[u8]) -> Result<(Self, usize), ()> {
        if buf.len() == 0 {
            return Err(());
        } else {
            return Ok((U8FixedWidth(buf[0]), 1));
        }
    }
}

/// A `u8` that uses a variable-width homomorphic encoding.
/// 
/// The encoding of a `u8` `n` consists of `n` times the byte `0x02`, followed by the single byte `0x01`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct U8VariableWidth(pub u8);

impl Dimension for U8VariableWidth {
    const HOMOMORPHIC_ENCODING_MAX_LENGTH: usize = 256;

    const IS_FIXED_WIDTH_ENCODING: bool = false;

    fn homomorphic_encode(&self, buf: &mut [u8]) -> usize {
        let n = self.0 as usize;
        for i in 0..n {
            buf[i] = 2;
        }
        buf[n] = 1;

        return n + 1;
    }

    fn homomorphic_decode(buf: &[u8]) -> Result<(Self, usize), ()> {
        let mut i = 0;
        while buf[i] != 1 {
            if i >= 256 {
                return Err(());
            }

            if buf[i] == 2 {
                i += 1;
            } else {
                return Err(());
            }
        }

        return Ok((Self(i as u8), i + 1));
    }
}