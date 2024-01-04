use crate::prelude::*;
use crate::{Deserialize, Serialize};
use tiny_serde_macros::{Deserialize, Serialize};

#[test]
fn proc_macros() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Foo {
        a: bool,
        b: u16,
    }

    const NUM: u16 = 0xff;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[repr(u16)]
    enum Eenie {
        A = 0xde,
        B,
        C,
        D = NUM,
        E,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[repr(u8)]
    enum Meenie {
        A,
        B { val: bool } = 0x10,
        C(u16),
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Bar {
        something: u16,
        foo: Foo,
        other: Eenie,
        another: Meenie,
        lastly: Meenie,
    }

    let bar = Bar {
        something: 0x10,
        foo: Foo { a: false, b: 0x100 },
        other: Eenie::B,
        another: Meenie::C(300),
        lastly: Meenie::A,
    };

    let buf = bar.clone().serialize();

    assert_eq!(
        buf,
        [0x0, 0x10, 0x0, 0x1, 0x0, 0x00, 0xdf, 0x11, 0x1, 0x2c, 0x0, 0x0, 0x0]
    );

    if let Some(debar) = Bar::deserialize(buf) {
        assert_eq!(bar, debar);
    }
}
