pub fn create_debugger_script(fn_name: &String, debugger_commands: &Vec<&str>) -> String {
    let mut debugger_script = String::new();

    // Add an inital breakpoint for the test function.
    debugger_script.push_str(format!("bm *!*::{}\n", fn_name).as_str());

    // Run the debugger to the start of the test.
    debugger_script.push_str("g\n");

    // Add the user specified breakpoints.
    debugger_script.push_str("bm *!*::__break \"gu\"\n");
    debugger_script.push_str("bl\n");

    // Run the debugger to the first user set breakpoint.
    debugger_script.push_str("g\n");

    for (i, debugger_comamand) in debugger_commands.iter().enumerate() {
        debugger_script.push_str(format!(".echo start_debugger_command_{}\n", i).as_str());
        debugger_script.push_str(format!("{}\n", debugger_comamand).as_str());
        debugger_script.push_str(format!(".echo end_debugger_command_{}\n", i).as_str());
    }

    // Be sure to quit the debugger after running all commands.
    debugger_script.push_str("qd\n");

    debugger_script
}

#[test]
pub fn test_debugger_script_empty() {
    let test_name = String::from("test1");
    let debugger_commands = vec![];
    let debugger_script = create_debugger_script(&test_name, &debugger_commands);
    let expected = r#"bm *!*::test1
g
bm *!*::__break "gu"
bl
g
qd
"#;

    assert_eq!(expected.to_string(), debugger_script);
}

#[test]
pub fn test_debugger_script() {
    let test_name = String::from("test1");
    let debugger_commands = vec!["dv", "g", ".nvlist"];
    let debugger_script = create_debugger_script(&test_name, &debugger_commands);
    let expected = r#"bm *!*::test1
g
bm *!*::__break "gu"
bl
g
.echo start_debugger_command_0
dv
.echo end_debugger_command_0
.echo start_debugger_command_1
g
.echo end_debugger_command_1
.echo start_debugger_command_2
.nvlist
.echo end_debugger_command_2
qd
"#;

    assert_eq!(expected.to_string(), debugger_script);
}
