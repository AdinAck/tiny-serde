# tiny-serde

A very small, explicitly static, serialization and deserialization interface.

# no_std

This crate is intended for use in `no_std` environments.

# Usage

## Serialize

A type can be serialized to the form of 4 `u8`'s if it implements `Serialize<u8, 4>`:

```rust
impl Serialize<u8, 4> for u32 {
    fn serialize(self) -> [u8, 4] {
        self.to_ne_bytes()
    }
}
```

## Deserialize

A type can be deserialized from 4 `u8`'s if it implements `Deserialize<u8, 4>`:

```rust
impl Deserialize<u8, 4> for u32 {
    fn deserialize(data: [u8; 4]) -> Self {
        Self::from_ne_bytes()
    }
}
```

## TryDeserialize

A type *may* or *may not* be deserialized from 4 `u8`'s if it implements `TryDeserialize<u8, 4`:

> A type that implements `Deserialize` automatically implements `TryDeserialize`.