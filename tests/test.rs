use debugger_test::debugger_test;

#[inline(never)]
pub fn __break() {}

#[debugger_test(debugger = "cdb", commands = "", expected_statements = "")]
pub fn test_empty_commands() {
    __break();
}

#[debugger_test(debugger = "cdb", commands = ".nvlist", expected_statements = "")]
pub fn test_no_expectations() {
    __break();
}

#[debugger_test(
    debugger = "cdb",
    commands = r#"
dv
dx a
g
dv
dx a
g
dv
dx b
g
dv
dx b"#,
    expected_statements = r#"
a = 0n0
a = 0n5
b = 0n25
a = 0n5
b = 0n10"#
)]
pub fn test_commands_with_expectations() {
    let mut a = 0;
    __break();

    a += 5;
    assert_eq!(a, 5);
    __break();

    let mut b = 25;
    __break();

    b -= 15;
    assert_eq!(b, 10);
    __break();
}
