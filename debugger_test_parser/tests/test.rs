use debugger_test_parser::parse;

/// Verify that a test failed with a specific error message.
fn verify_expected_failure(result: anyhow::Result<()>, expected_err_msg: &str) {
    let error = result
        .expect_err(format!("Expected error message missing: `{}`.", expected_err_msg).as_str());
    assert_eq!(expected_err_msg, format!("{error}"));
}

/// Test parsing empty debugger output.
#[test]
fn test_parse_empty_output() {
    let output = String::from("");
    let expected_contents = Vec::with_capacity(0);
    parse(output, expected_contents).expect("able to parse output.");
}

/// Test parsing debugger output for a single command.
/// No expected content.
#[test]
fn test_parse_output_command() {
    let output = String::from(
        r#"
    dv
        var1 = 0
        var2 = { len = 3 }
        var3 = { struct }
    "#,
    );

    let expected_contents = Vec::with_capacity(0);
    parse(output, expected_contents).expect("able to parse output.");
}

/// Test parsing a single debugger output.
/// Verify expected content.
#[test]
fn test_verify_output_command() {
    let output = String::from(
        r#"
    dv
        var1 = 0
        var2 = { len = 3 }
        var3 = { struct }
    "#,
    );

    let expected_contents = vec!["var1 = 0"];
    parse(output, expected_contents).expect("able to parse output.");
}

#[test]
fn test_trim_expected_contents() {
    let output = String::from(
        r#"
    dv
        var1 = 0
        var2 = { len = 3 }
        var3 = { struct }
    "#,
    );

    let expected_contents = vec![
        "        var1 = 0"
    ];
    parse(output, expected_contents).expect("able to parse output.");
}

/// Test parsing debugger output for mutliple commands.
/// Verify expected content.
#[test]
fn test_verify_output_multiple_commands() {
    let output = String::from(
        r#"
    .nvlist
        a.exe (embedded NatVis "path\to\foo.natvis")
    dx point_a
    point_a          : (0, 0) [Type: foo::Point]
        [<Raw View>]     [Type: foo::Point]
        [x]              : 0 [Type: int]
        [y]              : 0 [Type: int]
    dx point_b
    point_b          : (5, 8) [Type: foo::Point]
        [<Raw View>]     [Type: foo::Point]
        [x]              : 5 [Type: int]
        [y]              : 8 [Type: int]
    dx line
    line             : ((0, 0), (5, 8)) [Type: foo::Line]
        [<Raw View>]     [Type: foo::Line]
        [a]              : (0, 0) [Type: foo::Point]
        [b]              : (5, 8) [Type: foo::Point]
    dx person
    person           : "Person A" is 10 years old. [Type: foo::Person]
        [<Raw View>]     [Type: foo::Person]
        [name]           : "Person A" [Type: alloc::string::String]
        [age]            : 10 [Type: int]
    "#,
    );

    let expected_contents = vec![
        r#"pattern:a\.exe \(embedded NatVis ".*foo\.natvis"\)"#,
        "point_a          : (0, 0) [Type: foo::Point]",
        "[x]              : 0 [Type: int]",
        "person           : \"Person A\" is 10 years old. [Type: foo::Person]",
        "[name]           : \"Person A\" [Type: alloc::string::String]",
    ];
    parse(output, expected_contents).expect("able to parse output.");
}

/// Test expected content not found in debugger output due to incorrect ordering.
/// Parsing fails.
#[test]
fn test_err_expected_string_not_found() {
    let output = String::from(
        r#"
    .nvlist
        a.exe (embedded NatVis "path\to\foo.natvis")
    dx point_a
    point_a          : (0, 0) [Type: foo::Point]
        [<Raw View>]     [Type: foo::Point]
        [x]              : 0 [Type: int]
        [y]              : 0 [Type: int]
    dx point_b
    point_b          : (5, 8) [Type: foo::Point]
        [<Raw View>]     [Type: foo::Point]
        [x]              : 5 [Type: int]
        [y]              : 8 [Type: int]
    dx line
    line             : ((0, 0), (5, 8)) [Type: foo::Line]
        [<Raw View>]     [Type: foo::Line]
        [a]              : (0, 0) [Type: foo::Point]
        [b]              : (5, 8) [Type: foo::Point]
    dx person
    person           : "Person A" is 10 years old. [Type: foo::Person]
        [<Raw View>]     [Type: foo::Person]
        [name]           : "Person A" [Type: alloc::string::String]
        [age]            : 10 [Type: int]
    "#,
    );

    let expected_contents = vec![
        "person           : \"Person A\" is 10 years old. [Type: foo::Person]",
        "point_a          : (0, 0) [Type: foo::Point]",
    ];

    let expected_err_msg = "Unable to find expected content in the debugger output. Missing line: `point_a          : (0, 0) [Type: foo::Point]`";
    verify_expected_failure(parse(output, expected_contents), expected_err_msg);
}

/// Test expected pattern not found in debugger output due to incorrect ordering.
/// Parsing fails.
#[test]
fn test_err_expected_pattern_not_found() {
    let output = String::from(
        r#"
    .nvlist
        a.exe (embedded NatVis "path\to\foo.natvis")
    dx point_a
    point_a          : (0, 0) [Type: foo::Point]
        [<Raw View>]     [Type: foo::Point]
        [x]              : 0 [Type: int]
        [y]              : 0 [Type: int]
    dx point_b
    point_b          : (5, 8) [Type: foo::Point]
        [<Raw View>]     [Type: foo::Point]
        [x]              : 5 [Type: int]
        [y]              : 8 [Type: int]
    dx line
    line             : ((0, 0), (5, 8)) [Type: foo::Line]
        [<Raw View>]     [Type: foo::Line]
        [a]              : (0, 0) [Type: foo::Point]
        [b]              : (5, 8) [Type: foo::Point]
    dx person
    person           : "Person A" is 10 years old. [Type: foo::Person]
        [<Raw View>]     [Type: foo::Person]
        [name]           : "Person A" [Type: alloc::string::String]
        [age]            : 10 [Type: int]
    "#,
    );

    let expected_contents = vec![
        "point_a          : (0, 0) [Type: foo::Point]",
        r#"pattern:a\.exe \(embedded NatVis ".*foo\.natvis"\)"#,
    ];

    let expected_err_msg = "Unable to find expected content in the debugger output. Found 0 matches for pattern: `a\\.exe \\(embedded NatVis \".*foo\\.natvis\"\\)`";
    verify_expected_failure(parse(output, expected_contents), expected_err_msg);
}

/// Test expected pattern is not a valid regex.
/// Parsing fails.
#[test]
fn test_err_expected_pattern_not_valid() {
    let output = String::from(
        r#"
    .nvlist
        a.exe (embedded NatVis "path\to\foo.natvis")
    dv
        vec { len = 5 }
        vec2 { len = 1 }
    "#,
    );

    let expected_contents = vec![r#"pattern:vec2 { len = 1 }"#];

    let expected_err_msg = r#"Invalid regex pattern: vec2 { len = 1 }
regex parse error:
    vec2 { len = 1 }
           ^
error: repetition quantifier expects a valid decimal"#;
    verify_expected_failure(parse(output, expected_contents), expected_err_msg);
}
