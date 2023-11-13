#![no_std]

pub trait TryDeserialize<T, const N: usize>: Sized {
    fn try_deserialize(data: [T; N]) -> Option<Self>;
}

pub trait Deserialize<T, const N: usize>: Sized + TryDeserialize<T, N> {
    fn deserialize(data: [T; N]) -> Self;
}

impl<T, U, const N: usize> TryDeserialize<U, N> for T
where
    T: Deserialize<U, N>
{
    fn try_deserialize(data: [U; N]) -> Option<Self> {
        Some(Self::deserialize(data))
    }
}

pub trait Serialize<T, const N: usize>: Sized {
    fn serialize(self) -> [T; N];
}

macro_rules! primitive_ser_impls {
    ( $SER_TYPE:ty, $( $TAR_TYPE:ty: $SIZE:expr ),+ ) => {
        $(
            impl Deserialize<$SER_TYPE, $SIZE> for $TAR_TYPE {
                fn deserialize(data: [$SER_TYPE; $SIZE]) -> Self {
                    <$TAR_TYPE>::from_be_bytes(data)
                }
            }

            impl Serialize<$SER_TYPE, $SIZE> for $TAR_TYPE {
                fn serialize(self) -> [$SER_TYPE; $SIZE] {
                    <$TAR_TYPE>::to_be_bytes(self)
                }
            }
        )+
    };
}

primitive_ser_impls! {
    u8,
    u8: 1,
    u16: 2,
    u32: 4,
    u64: 8,
    f32: 4,
    f64: 8
}

#[cfg(test)]
mod tests {

}
