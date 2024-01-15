#![no_std]

#[cfg(test)]
mod tests;

#[cfg(feature = "derive")]
pub use macros::Serialize;

/// Records the size of a type intended for use as
/// a serialization medium.
pub trait TinySerDeSized {
    const SIZE: usize;
}

impl<const N: usize> TinySerDeSized for [u8; N] {
    const SIZE: usize = N;
}

/// Types that implement this trait can be serialized
/// to and from a serialization medium (usually bytes
/// in the form of [u8; N]).
pub trait Serialize: Sized {
    type Serialized: TinySerDeSized;
    type Error;

    /// Serializes the passed instance of this type (move).
    fn serialize(self) -> Self::Serialized;
    /// Creates an instance of this type; deserialized from the provided data (move).
    fn deserialize(data: Self::Serialized) -> Result<Self, Self::Error>;
}

#[doc(hidden)]
macro_rules! primitive_ser_impls {
    ( $( $TAR_TYPE:ty: $SIZE:expr ),+ ) => {
        $(
            impl Serialize for $TAR_TYPE {
                type Serialized = [u8; $SIZE];
                type Error = ();

                fn serialize(self) -> Self::Serialized {
                    <$TAR_TYPE>::to_be_bytes(self)
                }

                fn deserialize(data: Self::Serialized) -> Result<Self, Self::Error> {
                    Ok(<$TAR_TYPE>::from_be_bytes(data))
                }
            }
        )+
    };
}

primitive_ser_impls! {
    u8: 1,
    i8: 1,
    u16: 2,
    i16: 2,
    u32: 4,
    i32: 4,
    u64: 8,
    i64: 8,
    f32: 4,
    f64: 8
}

impl Serialize for bool {
    type Serialized = [u8; 1];
    type Error = ();

    fn serialize(self) -> Self::Serialized {
        (self as u8).serialize()
    }

    fn deserialize(data: Self::Serialized) -> Result<Self, Self::Error> {
        match u8::deserialize(data)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(()),
        }
    }
}
