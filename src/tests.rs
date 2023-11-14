use tiny_serde_macros::Serialize;

#[test]
fn proc_macros() {
    #[derive(Serialize)]
    struct Test {
        a: u8,
        b: u16
    }

    Test { a: 0, b: 1 };
}