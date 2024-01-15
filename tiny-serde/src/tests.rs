use crate::Serialize;
#[cfg(feature = "derive")]
use crate::TinySerDeSized;

#[test]
fn basic() {
    assert_eq!(128u8.serialize(), [0x80]);
    assert_eq!(
        u32::deserialize([0xde, 0xad, 0xbe, 0xef]),
        Ok(3_735_928_559)
    );
}

#[cfg(feature = "derive")]
#[test]
fn derive() {
    #[derive(Debug, Clone, PartialEq, Serialize)]
    struct Foo {
        a: bool,
        b: u16,
    }

    const NUM: u16 = 0xff;

    #[derive(Debug, Clone, PartialEq, Serialize)]
    #[repr(u16)]
    enum Eenie {
        A = 0xde,
        B,
        C,
        D = NUM,
        E,
    }

    #[derive(Debug, Clone, PartialEq, Serialize)]
    #[repr(u8)]
    enum Meenie {
        A,
        B { val: bool } = 0x10,
        C(u16),
    }

    #[derive(Debug, Clone, PartialEq, Serialize)]
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

    // testing the use of the "Serialized" associated type
    #[allow(unused_assignments)]
    let mut test_bar = <Bar as Serialize>::Serialized::default();
    test_bar = [
        0x0, 0x10, 0x0, 0x1, 0x0, 0x00, 0xdf, 0x11, 0x1, 0x2c, 0x0, 0x0, 0x0,
    ];

    assert_eq!(buf, test_bar);

    if let Ok(debar) = Bar::deserialize(buf) {
        assert_eq!(bar, debar);
    }
}
