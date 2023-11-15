# tiny-serde

A statically determined serialization and deserialization system for sized types.

## no_std

This crate is intended for use in `no_std` environments.

# Usage

This crate creates two traits: `Serialize` and `Deserialize`.

They are extremely simple:

```rust
pub trait Serialize<const N: usize>: Sized {
    fn serialize(self) -> [u8; N];
}
```

```rust
pub trait Deserialize<const N: usize>: Sized {
    fn deserialize(data: [u8; N]) -> Option<Self>;
}
```

As you can see, since these traits are restricted to sized types, the size of the serialized representation (`N`) is known as well.

Two convenience derive macros `Serialize` and `Deserialize` are provided and can be used like so:

```rust
#[derive(Serialize, Deserialize)]
#[repr(u16)]
enum Foo {
    A,
    B = 0xde,
    C
}

#[derive(Serialize, Deserialize)]
struct Bar {
    a: u8,
    foo: Foo,
    b: u16
}
```

> Since `Foo` implements `Serialize` and `Deserialize`, `Bar` can as well.

# Design Considerations

## Safety

No `unsafe` blocks are used in this crate. The derive macros cannot generate unsafe code. Worst case their output will just not compile.

## Note

The derive macros are a **zero-cost abstraction**. They employ constant evaluation to generate static code that extracts bytes from inner structures and places it into the result array. No counting pointer, no runtime checks (other than that of `array::copy_from_slice(...)`), direct insertion only.