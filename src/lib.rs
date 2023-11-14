#![no_std]

#[cfg(test)]
mod tests;

pub trait Serialize<T>: Sized {
    fn serialize(self) -> T;
}

pub trait TryDeserialize<T>: Sized {
    fn try_deserialize(data: T) -> Option<Self>;
}

pub trait Deserialize<T>: Sized + TryDeserialize<T> {
    fn deserialize(data: T) -> Self;
}

pub trait TryDeserializeIter<I>: Sized
where
    I: Iterator
{
    fn try_deserialize(iter: &mut I) -> Option<Self>;
}

impl<T, U> TryDeserialize<U> for T
where
    T: Deserialize<U>
{
    fn try_deserialize(data: U) -> Option<Self> {
        Some(Self::deserialize(data))
    }
}

macro_rules! primitive_ser_impls {
    ( $SER_TYPE:ty, $( $TAR_TYPE:ty: $SIZE:expr ),+ ) => {
        $(
            impl Serialize<[$SER_TYPE; $SIZE]> for $TAR_TYPE {
                fn serialize(self) -> [$SER_TYPE; $SIZE] {
                    <$TAR_TYPE>::to_be_bytes(self)
                }
            }

            impl Deserialize<[$SER_TYPE; $SIZE]> for $TAR_TYPE {
                fn deserialize(data: [$SER_TYPE; $SIZE]) -> Self {
                    <$TAR_TYPE>::from_be_bytes(data)
                }
            }

            impl TryDeserialize<&[$SER_TYPE]> for $TAR_TYPE {
                fn try_deserialize(data: &[$SER_TYPE]) -> Option<Self> {
                    Some(<$TAR_TYPE>::from_be_bytes(data.try_into().ok()?))
                }
            }

            impl<I> TryDeserializeIter<I> for $TAR_TYPE
            where
                I: Iterator<Item = $SER_TYPE>,
            {
                fn try_deserialize(iter: &mut I) -> Option<Self> {
                    Some(<$TAR_TYPE>::deserialize([iter.next()?; $SIZE]))
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
