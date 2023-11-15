#![no_std]

pub mod prelude;
#[cfg(test)]
mod tests;

use prelude::*;

pub trait Serialize<const N: usize>: Sized {
    fn serialize(self) -> [u8; N];
}

pub trait Deserialize<const N: usize>: Sized {
    fn deserialize(data: [u8; N]) -> Option<Self>;
}

#[doc(hidden)]
macro_rules! primitive_ser_impls {
    ( $( $TAR_TYPE:ty: $SIZE:expr ),+ ) => {
        $(
            impl _TinySerSized for $TAR_TYPE {
                const SIZE: usize = $SIZE;
            }

            impl _TinyDeSized for $TAR_TYPE {
                const SIZE: usize = $SIZE;
            }

            impl Serialize<$SIZE> for $TAR_TYPE {
                fn serialize(self) -> [u8; $SIZE] {
                    <$TAR_TYPE>::to_be_bytes(self)
                }
            }

            impl Deserialize<$SIZE> for $TAR_TYPE {
                fn deserialize(data: [u8; $SIZE]) -> Option<Self> {
                    Some(<$TAR_TYPE>::from_be_bytes(data))
                }
            }
        )+
    };
}

primitive_ser_impls! {
    u8: 1,
    u16: 2,
    u32: 4,
    u64: 8,
    f32: 4,
    f64: 8
}
