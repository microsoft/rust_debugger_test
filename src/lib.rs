mod debugger;
mod debugger_script;

use std::str::FromStr;

use debugger::DebuggerType;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parse, Token};

use crate::debugger_script::create_debugger_script;

struct DebuggerTest {
    debugger: String,
    commands: String,
    expected_statements: String,
}

impl Parse for DebuggerTest {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let debugger_meta = input.parse::<syn::MetaNameValue>()?;
        let debugger = if debugger_meta.path.is_ident("debugger") {
            match debugger_meta.lit {
                syn::Lit::Str(lit_str) => lit_str.value(),
                _ => {
                    return Err(input.error("Expected a literal string for the value of `debugger`"))
                }
            }
        } else {
            return Err(input.error("Expected value `debugger`"));
        };

        input.parse::<Token![,]>()?;

        let commands_meta = input.parse::<syn::MetaNameValue>()?;
        let commands = if commands_meta.path.is_ident("commands") {
            match commands_meta.lit {
                syn::Lit::Str(lit_str) => lit_str.value(),
                _ => {
                    return Err(input.error("Expected a literal string for the value of `commands`"))
                }
            }
        } else {
            return Err(input.error("Expected value `commands`"));
        };

        input.parse::<Token![,]>()?;

        let expected_statements_meta = input.parse::<syn::MetaNameValue>()?;
        let expected_statements = if expected_statements_meta
            .path
            .is_ident("expected_statements")
        {
            match expected_statements_meta.lit {
                syn::Lit::Str(lit_str) => lit_str.value(),
                _ => {
                    return Err(input
                        .error("Expected a literal string for the value of `expected_statements`"))
                }
            }
        } else {
            return Err(input.error("Expected value `expected_statements`"));
        };

        Ok(DebuggerTest {
            debugger,
            commands,
            expected_statements,
        })
    }
}

#[proc_macro_attribute]
pub fn debugger_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let invoc = match syn::parse::<DebuggerTest>(attr) {
        Ok(s) => s,
        Err(e) => return e.to_compile_error().into(),
    };

    let item = match syn::parse::<syn::Item>(item) {
        Ok(s) => s,
        Err(e) => return e.to_compile_error().into(),
    };

    let func = match item {
        syn::Item::Fn(ref f) => f,
        _ => panic!("must be attached to a function"),
    };

    let debugger_commands = &invoc
        .commands
        .trim()
        .lines()
        .into_iter()
        .map(|line| line.trim())
        .collect::<Vec<&str>>();

    let debugger_type = DebuggerType::from_str(invoc.debugger.as_str()).expect(
        format!(
            "debugger `{}` must be a valid debugger option.",
            invoc.debugger.as_str()
        )
        .as_str(),
    );
    let debugger_executable_path = debugger::get_debugger(&debugger_type);

    let fn_name = func.sig.ident.to_string();
    let fn_ident = format_ident!("{}", fn_name);
    let test_fn_name = format!("{}__{}", fn_name, debugger_type.to_string());
    let test_fn_ident = format_ident!("{}", test_fn_name);

    let debugger_script_contents = create_debugger_script(&fn_name, debugger_commands);

    // Trim all whitespace and remove any empty lines.
    let expected_statements = &invoc
        .expected_statements
        .trim()
        .lines()
        .collect::<Vec<&str>>();

    // Create the cli for the given debugger.
    let (debugger_command_line, cfg_attr) = match debugger_type {
        DebuggerType::Cdb => {
            let debugger_path = debugger_executable_path.to_string_lossy().to_string();
            let command_line = quote!(
                match std::process::Command::new(#debugger_path)
                    .stdout(std::process::Stdio::from(debugger_stdout_file))
                    .stderr(std::process::Stdio::from(debugger_stderr_file))
                    .arg("-pd")
                    .arg("-p")
                    .arg(pid.to_string())
                    .arg("-cf")
                    .arg(&debugger_script_path)
                    .spawn() {
                        Ok(child) => child,
                        Err(error) => {
                            return Err(std::boxed::Box::from(format!("Failed to launch CDB: {}\n", error.to_string())));
                        }
                }
            );

            // cdb is only supported on Windows.
            let cfg_attr = quote!(
                #[cfg_attr(not(target_os = "windows"), ignore = "test only runs on windows platforms.")]
            );

            (command_line, cfg_attr)
        }
    };

    // Create the test function that will launch the debugger and run debugger commands.
    let mut debugger_test_fn = proc_macro::TokenStream::from(quote!(
        #[test]
        #cfg_attr
        fn #test_fn_ident() -> std::result::Result<(), Box<dyn std::error::Error>> {
            use std::io::Read;
            use std::io::Write;

            let pid = std::process::id();
            let current_exe_filename = std::env::current_exe()?.file_stem().expect("must have a valid file name").to_string_lossy().to_string();

            // Create a temporary file to store the debugger script to run.
            let debugger_script_filename = format!("{}_{}.debugger_script", current_exe_filename, #test_fn_name);
            let debugger_script_path = std::env::temp_dir().join(debugger_script_filename);

            // Write the contents of the debugger script to a new file.
            let mut debugger_script = std::fs::File::create(&debugger_script_path)?;
            writeln!(debugger_script, #debugger_script_contents)?;

            // Create a temporary file to store the stdout and stderr from the debugger output.
            let debugger_stdout_path = debugger_script_path.with_extension("debugger_out");
            let debugger_stderr_path = debugger_script_path.with_extension("debugger_err");

            let debugger_stdout_file = std::fs::File::create(&debugger_stdout_path)?;
            let debugger_stderr_file = std::fs::File::create(&debugger_stderr_path)?;

            // Start the debugger and run the debugger commands.
            let mut child = #debugger_command_line;

            // Wait for the debugger to attach
            // On Windows, use the IsDebuggerPresent API to check if a debugger is present
            // for the current process. https://docs.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-isdebuggerpresent
            #[cfg(windows)]
            extern "stdcall" {
                fn IsDebuggerPresent() -> i32;
            };
            #[cfg(windows)]
            unsafe {
                while IsDebuggerPresent() == 0 {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }

            // If not on Windows, wait 5 seconds for the debugger to attach.
            #[cfg(not(windows))]
            std::thread::sleep(std::time::Duration::from_secs(3));

            // Call the test function.
            #fn_ident();

            // Wait for the debugger to exit.
            std::thread::sleep(std::time::Duration::from_secs(3));

            // If debugger has not already quit, force quit the debugger.
            let mut debugger_stdout = String::new();
            match child.try_wait()? {
                Some(status) => {
                    // Bail early if the debugger process didn't execute successfully.
                    let mut debugger_stdout_file = std::fs::File::open(&debugger_stdout_path)?;
                    debugger_stdout_file.read_to_string(&mut debugger_stdout)?;

                    if !status.success() {
                        let mut debugger_stderr = String::new();
                        let mut debugger_stderr_file = std::fs::File::open(&debugger_stderr_path)?;
                        debugger_stderr_file.read_to_string(&mut debugger_stderr)?;
                        return Err(std::boxed::Box::from(format!("Debugger failed with {}.\n{}\n{}\n", status, debugger_stderr, debugger_stdout)));
                    }

                    println!("Debugger stdout:\n{}\n", &debugger_stdout);

                    // Verify the expected contents of the debugger output.
                    let expected_statements = vec![#(#expected_statements),*];
                    debugger_test_parser::parse(debugger_stdout, expected_statements)?;
                },
                None => {
                    // Force kill the debugger process if it has not exited yet.
                    println!("killing debugger process.");
                    child.kill().expect("debugger has been running for too long");

                    let mut debugger_stdout_file = std::fs::File::open(&debugger_stdout_path)?;
                    debugger_stdout_file.read_to_string(&mut debugger_stdout)?;
                    println!("Debugger stdout:\n{}\n", &debugger_stdout);
                }
            }

            #[cfg(windows)]
            unsafe {
                while IsDebuggerPresent() == 1 {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }

            #[cfg(not(windows))]
            std::thread::sleep(std::time::Duration::from_secs(3));

            Ok(())
        }
    ));

    debugger_test_fn.extend(proc_macro::TokenStream::from(item.to_token_stream()).into_iter());
    debugger_test_fn
}
