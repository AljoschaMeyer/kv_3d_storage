use core::cmp::{Ordering, Ordering::*};
use core::future::Future;

/// A point in a 3d space. Note that this struct does *not* implement `Ord`. Instead it provides three functions for three possible choices of total orderings: [`cmp_xyz`](Self::cmp_xyz), [`cmp_yzx`](Self::cmp_yzx), and [`cmp_zxy`](Self::cmp_zxy). This is to make sure that any comparisons explicitly select an ordering.
///
/// The three dimensions have types `X`, `Y`, and `Z`.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Point3d<X, Y, Z>
where
    X: Dimension,
    Y: Dimension,
    Z: Dimension,
{
    pub x: X,
    pub y: Y,
    pub z: Z,
}

impl<X: Dimension, Y: Dimension, Z: Dimension> Point3d<X, Y, Z> {
    /// Compare by x dimension first, using the y dimension as a tiebreaker, and using the z dimension as the final tiebreaker.
    pub fn cmp_xyz(&self, other: &Self) -> Ordering {
        let x_cmp = self.x.cmp(&other.x);
        match x_cmp {
            Equal => {
                let y_cmp = self.y.cmp(&other.y);
                match y_cmp {
                    Equal => return self.z.cmp(&other.z),
                    _ => return y_cmp,
                }
            }
            _ => return x_cmp,
        }
    }

    /// Compare by y dimension first, using the z dimension as a tiebreaker, and using the x dimension as the final tiebreaker.
    pub fn cmp_yzx(&self, other: &Self) -> Ordering {
        let y_cmp = self.y.cmp(&other.y);
        match y_cmp {
            Equal => {
                let z_cmp = self.z.cmp(&other.z);
                match z_cmp {
                    Equal => return self.x.cmp(&other.x),
                    _ => return z_cmp,
                }
            }
            _ => return y_cmp,
        }
    }

    /// Compare by z dimension first, using the x dimension as a tiebreaker, and using the y dimension as the final tiebreaker.
    pub fn cmp_zxy(&self, other: &Self) -> Ordering {
        let z_cmp = self.z.cmp(&other.z);
        match z_cmp {
            Equal => {
                let x_cmp = self.x.cmp(&other.x);
                match x_cmp {
                    Equal => return self.y.cmp(&other.y),
                    _ => return x_cmp,
                }
            }
            _ => return z_cmp,
        }
    }

    /// Return the maximum length of any [xyz-encoding](Self::encode_xyz).
    pub const fn max_encoding_len_xyz() -> usize {
        return X::HOMOMORPHIC_ENCODING_MAX_LENGTH
            + if X::IS_FIXED_WIDTH_ENCODING { 0 } else { 2 }
            + Y::HOMOMORPHIC_ENCODING_MAX_LENGTH
            + if Y::IS_FIXED_WIDTH_ENCODING { 0 } else { 2 }
            + Z::HOMOMORPHIC_ENCODING_MAX_LENGTH;
    }

    /// Encode a [`Point3d`](Self) with an encoding that is homomorphic to the [xyz ordering](Self::cmp_xyz), and return how long the produced encoding is.
    ///
    /// Panic if the encoding is longer than the given slice. To prevent this, ensure the slice has a length of at least [`max_encoding_len_xyz`](Self::max_encoding_len_xyz).
    pub fn encode_xyz(&self, buf: &mut [u8]) -> usize {
        let mut len = 0;

        len += self.x.homomorphic_encode(&mut buf[len..]);
        if !X::IS_FIXED_WIDTH_ENCODING {
            buf[len] = 0;
            buf[len + 1] = 0;
            len += 2;
        }

        len += self.y.homomorphic_encode(&mut buf[len..]);
        if !Y::IS_FIXED_WIDTH_ENCODING {
            buf[len] = 0;
            buf[len + 1] = 0;
            len += 2;
        }

        len += self.z.homomorphic_encode(&mut buf[len..]);

        return len;
    }

    /// Decode the [xyz encoding](Self::encode_xyz) from a slice. On success, return the decoded value, and the number of bytes that were decoded.
    pub fn decode_xyz(buf: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;

        let (x, x_len) = X::homomorphic_decode(&buf[offset..])?;
        offset += x_len;
        if !X::IS_FIXED_WIDTH_ENCODING {
            if buf[offset] != 0 || buf[offset + 1] != 0 {
                return Err(());
            } else {
                offset += 2;
            }
        }

        let (y, y_len) = Y::homomorphic_decode(&buf[offset..])?;
        offset += y_len;
        if !Y::IS_FIXED_WIDTH_ENCODING {
            if buf[offset] != 0 || buf[offset + 1] != 0 {
                return Err(());
            } else {
                offset += 2;
            }
        }

        let (z, z_len) = Z::homomorphic_decode(&buf[offset..])?;
        offset += z_len;

        return Ok((Point3d { x, y, z }, offset));
    }

    /// Return the maximum length of any [yzx-encoding](Self::encode_xyz).
    pub const fn max_encoding_len_yzx() -> usize {
        return Y::HOMOMORPHIC_ENCODING_MAX_LENGTH
            + if Y::IS_FIXED_WIDTH_ENCODING { 0 } else { 2 }
            + Z::HOMOMORPHIC_ENCODING_MAX_LENGTH
            + if Z::IS_FIXED_WIDTH_ENCODING { 0 } else { 2 }
            + X::HOMOMORPHIC_ENCODING_MAX_LENGTH;
    }

    /// Encode a [`Point3d`](Self) with an encoding that is homomorphic to the [yzx ordering](Self::cmp_yzx), and return how long the produced encoding is.
    ///
    /// Panic if the encoding is longer than the given slice. To prevent this, ensure the slice has a length of at least [`max_encoding_len_yzx`](Self::max_encoding_len_yzx).
    pub fn encode_yzx(&self, buf: &mut [u8]) -> usize {
        let mut len = 0;

        len += self.y.homomorphic_encode(&mut buf[len..]);
        if !Y::IS_FIXED_WIDTH_ENCODING {
            buf[len] = 0;
            buf[len + 1] = 0;
            len += 2;
        }

        len += self.z.homomorphic_encode(&mut buf[len..]);
        if !Z::IS_FIXED_WIDTH_ENCODING {
            buf[len] = 0;
            buf[len + 1] = 0;
            len += 2;
        }

        len += self.x.homomorphic_encode(&mut buf[len..]);

        return len;
    }

    /// Decode the [yzx encoding](Self::encode_yzx) from a slice. On success, return the decoded value, and the number of bytes that were decoded.
    pub fn decode_yzx(buf: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;

        let (y, y_len) = Y::homomorphic_decode(&buf[offset..])?;
        offset += y_len;
        if !Y::IS_FIXED_WIDTH_ENCODING {
            if buf[offset] != 0 || buf[offset + 1] != 0 {
                return Err(());
            } else {
                offset += 2;
            }
        }

        let (z, z_len) = Z::homomorphic_decode(&buf[offset..])?;
        offset += z_len;
        if !Z::IS_FIXED_WIDTH_ENCODING {
            if buf[offset] != 0 || buf[offset + 1] != 0 {
                return Err(());
            } else {
                offset += 2;
            }
        }

        let (x, x_len) = X::homomorphic_decode(&buf[offset..])?;
        offset += x_len;

        return Ok((Point3d { x, y, z }, offset));
    }

    /// Return the maximum length of any [xyz-encoding](Self::encode_xyz).
    pub const fn max_encoding_len_zxy() -> usize {
        return Z::HOMOMORPHIC_ENCODING_MAX_LENGTH
            + if Z::IS_FIXED_WIDTH_ENCODING { 0 } else { 2 }
            + X::HOMOMORPHIC_ENCODING_MAX_LENGTH
            + if X::IS_FIXED_WIDTH_ENCODING { 0 } else { 2 }
            + Y::HOMOMORPHIC_ENCODING_MAX_LENGTH;
    }

    /// Encode a [`Point3d`](Self) with an encoding that is homomorphic to the [zxy ordering](Self::cmp_zxy), and return how long the produced encoding is.
    ///
    /// Panic if the encoding is longer than the given slice. To prevent this, ensure the slice has a length of at least [`max_encoding_len_zxy`](Self::max_encoding_len_zxy).
    pub fn encode_zxy(&self, buf: &mut [u8]) -> usize {
        let mut len = 0;

        len += self.z.homomorphic_encode(&mut buf[len..]);
        if !Z::IS_FIXED_WIDTH_ENCODING {
            buf[len] = 0;
            buf[len + 1] = 0;
            len += 2;
        }

        len += self.x.homomorphic_encode(&mut buf[len..]);
        if !X::IS_FIXED_WIDTH_ENCODING {
            buf[len] = 0;
            buf[len + 1] = 0;
            len += 2;
        }

        len += self.y.homomorphic_encode(&mut buf[len..]);

        return len;
    }

    /// Decode the [zxy encoding](Self::encode_zxy) from a slice.  On success, return the decoded value, and the number of bytes that were decoded.
    pub fn decode_zxy(buf: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;

        let (z, z_len) = Z::homomorphic_decode(&buf[offset..])?;
        offset += z_len;
        if !Z::IS_FIXED_WIDTH_ENCODING {
            if buf[offset] != 0 || buf[offset + 1] != 0 {
                return Err(());
            } else {
                offset += 2;
            }
        }

        let (x, x_len) = X::homomorphic_decode(&buf[offset..])?;
        offset += x_len;
        if !X::IS_FIXED_WIDTH_ENCODING {
            if buf[offset] != 0 || buf[offset + 1] != 0 {
                return Err(());
            } else {
                offset += 2;
            }
        }

        let (y, y_len) = Y::homomorphic_decode(&buf[offset..])?;
        offset += y_len;

        return Ok((Point3d { x, y, z }, offset));
    }
}

/// A type that can be used as a dimension of a [`Point3d`].
///
/// Must be totally ordered, and must provide an order-homomorphic [encoding function](https://willowprotocol.org/specs/encodings/index.html#encoding_function), that is., comparing encodings lexicographically must coincide with the total order on the dimension.
pub trait Dimension: Ord + Sized {
    /// The maximum length of any [homomorphic encoding](Self::homomorphic_encode).
    const HOMOMORPHIC_ENCODING_MAX_LENGTH: usize;

    /// Do the [homomorphic encodings](Self::homomorphic_encode) of all values have the same length? If this is `false`, then no encoding may contain two successive zero bytes (the combined encoding of a `3dPoint` will use two consecutive zero bytes to terminate variable-width encodings, so things will subtly break if the encodings contained consecutive zero bytes themselves).
    const IS_FIXED_WIDTH_ENCODING: bool;

    /// Encode `self` into a slice of at least `Self::HOMOMORPHIC_ENCODING_LENGTH` many bytes, and return how long the produced encoding is. The [encoding](https://willowprotocol.org/specs/encodings/index.html#encoding_function) must be order-homomorphic, that is: for any two values `v1` and `v2` with `v1 <= v2`, the encoding of `v1` must be lexicographically less than or equal to the encoding of `v2`. Further, if [`IS_FIXED_WIDTH_ENCODING`](Self::IS_FIXED_WIDTH_ENCODING) is `false`, then no encoding may contain two consecutive zero bytes.
    ///
    /// If the encoding is longer than the given slice, this function must panic.
    fn homomorphic_encode(&self, buf: &mut [u8]) -> usize;

    /// Decode the [homomorphic encoding](Self::homomorphic_encode) from a slice. On success, return the decoded value, and the number of bytes that were decoded.
    fn homomorphic_decode(buf: &[u8]) -> Result<(Self, usize), ()>;
}

/// A persistent storage backend that maps bytestrings keys to values of some type `V`, and allows for efficient access based on the lexicographic ordering of the keys.
pub trait BackEnd<V> {
    /// Type of errors that can occur when interacting with the backend.
    type Error;

    /// Insert a kv pair. Returns the old value for that key, if there was any.
    ///
    /// This need not be persisted to disk immediately, persistence may be delayed until [`flush`](Self::flush) is called. All subsequent method calls must incorporat the insertion though, even if it has not been persisted yet.
    fn insert(
        &mut self,
        key: &[u8],
        value: V,
    ) -> impl Future<Output = Result<Option<V>, Self::Error>>;

    /// Delete a kv pair. Returns the old value for that key, if there was any.
    ///
    /// This need not be persisted to disk immediately, persistence may be delayed until [`flush`](Self::flush) is called. All subsequent method calls must incorporat the deletion though, even if it has not been persisted yet.
    fn delete(&mut self, key: &[u8]) -> impl Future<Output = Result<Option<V>, Self::Error>>;

    /// Commit all mutations that have been performed so far to disk. When the Future is done, the changes are guaranteed to be persisted.
    fn flush(&mut self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Get the value associated with the given key, if there is any.
    fn get(&self, key: &[u8]) -> impl Future<Output = Result<Option<V>, Self::Error>>;

    /// Get the greatest kv pair whose key is less than or equal to the given key, if there is any.
    fn find_lte(&self, key: &[u8])
        -> impl Future<Output = Result<Option<(&[u8], V)>, Self::Error>>;

    /// Get the least kv pair whose key is greater than or equal to the given key, if there is any.
    fn find_gte(&self, key: &[u8])
        -> impl Future<Output = Result<Option<(&[u8], V)>, Self::Error>>;
}

// TODO batch/transaction