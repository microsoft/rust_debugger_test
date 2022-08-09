# debugger_test

Provides an easy way of integrating debugger specific tests into a crate.

This crate is responsible for generating the `#[debugger_test]` proc macro attribute.

## Usage

To use, add this crate and the `debugger_test_parser` as a dependency in your `Cargo.toml`.

This crate uses the `debugger_test_parser` to parse the output of the specified debugger
and verify all expected statements were found.

In order to set breakpoints, an `__break()` function will need to be defined and called
at each place the debugger should stop.

For example:

```rust
#[inline(never)]
fn __break() { }

#[debugger_test(
    debugger = "cdb",
    commands = r#"
.nvlist
dv
g"#,
    expected_statements = r#"
pattern:test\.exe .*\.natvis
a = 0n10
    "#)]
fn test() {
    let a = 10;
    __break();
}
```

The `#[debugger_test]` proc macro attribute has 3 required meta items which all take a string value:

1. debugger
2. commands
3. expected_statements

The `debugger` meta item expects the name of a supported debugger. Currently the only supported debugger is `cdb`.
This crate will try to find the specified debugger, first by testing if it is on the `PATH`. If the debugger is
not found, this crate will search the default installation directory for the debugger. Specifying an exact path
for which debugger to use is not currently supported.

The `commands` meta item expects a string of a debugger command to run. To run multiple commands, separate each
command by the new line character (`\n`).

The `expected_statements` meta item expects a string of output to verify in the debugger output.
Each statement should be separated by a new line character (`\n`).

For example:

```rust
#[debugger_test(
    debugger = "cdb",
    commands = "command1\ncommand2\ncommand3",
    expected_statements = "statement1\nstatement2\nstatement3")]
```

Using a multiline string is also supported:

```rust
#[debugger_test(
    debugger = "cdb",
    commands = r#"
command1
command2
command3"#,
    expected_statements = r#"
statement1
statement2
statement3"#)]
```

Pattern matching is also supported for a given `expected_statement`. Use the prefix, `pattern:` for the
expected statement. This is useful for ignoring debugger output that contain memory address and/or paths:

```rust
#[debugger_test(
    debugger = "cdb",
    commands = "command3",
    expected_statements = "pattern:abc.*")]
```

The `#[debugger_test]` proc macro attribute will generate a new test function that will be marked
with the `#[test]` attribute. This generated test function will add a suffix to the test name to ensure
the test is unique. In the example above, the proc macro attribute will generate the following function:

```rust
#[test]
fn test__cdb() {
    .....
    test();
    .....
}
```

The proc macro attribute will generate a test function that will do the following:

1. Launch the specified debugger
2. Attach the debugger to the current test executable process
3. Set breakpoints at all call sites of the `__break()` function
4. Run the debugger to the first breakpoint specified by the debugger
5. Run all of the user specified commands and exit the debugger
6. Parse the debugger output using the `debugger_test_parser` crate and verify all the `expected_statements` were found

Based on the debugger specified via the `#[debugger_test]` attribute, the path used to launch the debugger will
be one of the following:

1. If the environment variable, _debugger_type_ _DEBUGGER_DIR is set, i.e. `CDB_DEBUGGER_DIR`, the proc macro attribute will try to launch the debugger from this directory
2. The default installation directory for the given debugger if it exists at that path
3. Invoking the executable directly, i.e. `cdb` or `cdb.exe` depending on the OS

## Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit https://cla.opensource.microsoft.com.

When you submit a pull request, a CLA bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft 
trademarks or logos is subject to and must follow 
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
