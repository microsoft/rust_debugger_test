use std::env;
use std::ffi::OsString;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};

#[cfg(windows)]
pub static EXECUTABLE_EXTENSION: &str = ".exe";
#[cfg(not(windows))]
pub static EXECUTABLE_EXTENSION: &str = "";

#[derive(Debug, EnumIter)]
pub enum DebuggerType {
    Cdb,
}

impl Display for DebuggerType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let debugger_type = match self {
            DebuggerType::Cdb => "cdb",
        };
        write!(fmt, "{}", debugger_type)
    }
}

impl FromStr for DebuggerType {
    type Err = anyhow::Error;

    /// Attempts to parse a string into a DebuggerType
    fn from_str(s: &str) -> Result<DebuggerType, Self::Err> {
        let debugger = s.to_lowercase();
        for debugger_type in DebuggerType::iter() {
            if debugger == format!("{}", debugger_type) {
                return anyhow::Ok(debugger_type);
            }
        }
        anyhow::bail!("Invalid debugger type option: `{}`.", s)
    }
}

/// Find the CDB debugger by searching its default installation directory.
fn find_cdb() -> Option<OsString> {
    // Inspired by https://github.com/rust-lang/rust/blob/1.62.0/src/tools/compiletest/src/main.rs#L821
    let pf86 = env::var_os("ProgramFiles(x86)").or_else(|| env::var_os("ProgramFiles"))?;

    let cdb_arch = if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else {
        return None; // No compatible cdb.exe in the Windows 10 SDK
    };

    let mut path = PathBuf::new();
    path.push(pf86);
    path.push(r"Windows Kits\10\Debuggers");
    path.push(cdb_arch);
    path.push("cdb.exe");

    if !path.exists() {
        return None;
    }

    Some(path.into_os_string())
}

/// Get the debugger specified by the debugger_type parameter.
pub fn get_debugger(debugger_type: &DebuggerType) -> PathBuf {
    let debugger_executable = OsString::from(format!("{}{}", debugger_type, EXECUTABLE_EXTENSION));

    let debugger_env_dir = match debugger_type {
        DebuggerType::Cdb => env::var_os("CDB_DEBUGGER_DIR"),
    };

    // First check to see if the %debugger_type%_DEBUGGER_DIR environment variable is set.
    // If set, use this directory for all debugger invocations.
    // If not set, fallback to the default installation directory.
    // If the debugger is not found there, fallback to the current path.
    let debugger_executable_path = if let Some(debugger_env_path) = debugger_env_dir {
        PathBuf::from(debugger_env_path).join(debugger_executable)
    } else {
        match debugger_type {
            DebuggerType::Cdb => PathBuf::from(find_cdb().unwrap_or(debugger_executable)),
        }
    };

    debugger_executable_path
}

#[test]
#[cfg_attr(
    not(target_os = "windows"),
    ignore = "test only runs on windows platforms."
)]
fn test_find_cdb() {
    let result = find_cdb();
    assert!(result.is_some());

    let cdb = result.unwrap();
    let cdb_path = std::path::PathBuf::from(cdb.to_string_lossy().to_string());
    assert!(cdb_path.file_name().is_some());

    let cdb_exe = cdb_path.file_name().unwrap();
    assert_eq!("cdb.exe", cdb_exe);
}

#[test]
fn test_get_debugger() {
    let debugger_type = DebuggerType::Cdb;
    let cdb_executable = format!("cdb{}", EXECUTABLE_EXTENSION);

    // Test setting the environment variable to find the debugger
    let cdb_debugger_dir = "debugger_path/debugger";
    env::set_var("CDB_DEBUGGER_DIR", cdb_debugger_dir);
    assert!(env::var_os("CDB_DEBUGGER_DIR").unwrap() == OsString::from("debugger_path/debugger"));

    let mut debugger_path = get_debugger(&debugger_type);
    let expected_path = PathBuf::from(cdb_debugger_dir).join(&cdb_executable);
    assert_eq!(expected_path, debugger_path);
    env::remove_var("CDB_DEBUGGER_DIR");

    debugger_path = get_debugger(&debugger_type);
    assert_eq!(
        cdb_executable,
        debugger_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    );
}

#[test]
fn test_debugger_type_from_str() {
    assert!(DebuggerType::from_str("cdb").is_ok());

    let gdb_debugger_type = DebuggerType::from_str("gdb");
    assert!(gdb_debugger_type.is_err());
    assert_eq!(
        "Invalid debugger type option: `gdb`.",
        format!("{}", gdb_debugger_type.unwrap_err())
    );

    let mock_debugger_debugger_type = DebuggerType::from_str("mock debugger");
    assert!(mock_debugger_debugger_type.is_err());
    assert_eq!(
        "Invalid debugger type option: `mock debugger`.",
        format!("{}", mock_debugger_debugger_type.unwrap_err())
    );
}
