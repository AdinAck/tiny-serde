use crate::prelude::*;
use crate::{Deserialize, Serialize};
use tiny_serde_macros::{Deserialize, Serialize};

#[test]
fn proc_macros() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Foo {
        a: u8,
        b: u16,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[repr(u16)]
    enum Eenie {
        A = 0xde,
        B,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Bar {
        something: u16,
        foo: Foo,
        other: Eenie,
    }

    let bar = Bar {
        something: 0x10,
        foo: Foo { a: 0x2c, b: 0x100 },
        other: Eenie::B,
    };

    let buf = bar.clone().serialize();

    assert_eq!(buf, [0x0, 0x10, 0x2c, 0x1, 0x0, 0x00, 0xdf]);

    if let Some(debar) = Bar::deserialize(buf) {
        assert_eq!(bar, debar);
    }
}
