use std::env;
use std::ffi::OsString;
use std::fmt::Display;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use regex::Regex;
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
            if debugger == format!("{debugger_type}") {
                return anyhow::Ok(debugger_type);
            }
        }
        anyhow::bail!("Invalid debugger type option: `{}`.", s)
    }
}

#[derive(Debug)]
pub enum Debugger {
    Cdb(PathBuf),
}

impl Display for Debugger {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let (debugger_type, debugger_path) = match self {
            Debugger::Cdb(path) => ("cdb", path),
        };
        write!(fmt, "{}: {}", debugger_type, debugger_path.display())
    }
}

fn find_cdb() -> anyhow::Result<OsString> {
    let pf86 = env::var_os("ProgramFiles(x86)")
        .or_else(|| env::var_os("ProgramFiles"))
        .expect("msg");
    let cdb_arch = if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else {
        anyhow::bail!("No compatible cdb.exe in the Windows 10 SDK");
    };

    let mut path = PathBuf::new();
    path.push(pf86);
    path.push(r"Windows Kits\10\Debuggers");
    path.push(cdb_arch);
    path.push(r"cdb.exe");

    if !path.exists() {
        anyhow::bail!(
            "Unable to find cdb.exe at `{:?}`. Please ensure the debugger is installed.",
            path.display()
        );
    }

    Ok(path.into_os_string())
}

/// Get the debugger specified by the debugger_type parameter.
pub fn get_debugger(debugger_type: &DebuggerType) -> anyhow::Result<Debugger> {
    let mut debugger_executable =
        OsString::from(format!("{}{}", debugger_type, EXECUTABLE_EXTENSION));
    let version_arg = match debugger_type {
        DebuggerType::Cdb => "-version",
    };

    let result = Command::new(&debugger_executable).arg(version_arg).output();

    // First check to see if the debugger is on the path.
    let output = match result {
        Ok(output) => output,
        Err(error) => match error.kind() {
            io::ErrorKind::NotFound => {
                log::info!("Unable to find debugger `{:?}` on the Path. Searching default installation directory.", debugger_type);

                debugger_executable = match debugger_type {
                    DebuggerType::Cdb => find_cdb()?,
                };

                Command::new(&debugger_executable)
                    .arg(version_arg)
                    .output()?
            }
            error_kind => anyhow::bail!("{error_kind}"),
        },
    };

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Debugger `{:?}` failed with {:?}\n{}\n{}",
            &debugger_executable,
            output.status.code(),
            stdout,
            stderr
        );
    }

    let debugger = match debugger_type {
        DebuggerType::Cdb => Debugger::Cdb(PathBuf::from(&debugger_executable)),
    };

    Ok(debugger)
}

#[test]
#[cfg_attr(
    not(target_os = "windows"),
    ignore = "test only runs on windows platforms."
)]
pub fn test_find_cdb() {
    let result = find_cdb();
    assert!(result.is_ok());

    let cdb = result.unwrap();
    let cdb_path = std::path::PathBuf::from(cdb.to_string_lossy().to_string());
    assert!(cdb_path.file_name().is_some());

    let cdb_exe = cdb_path.file_name().unwrap();
    assert_eq!("cdb.exe", cdb_exe);
}

#[test]
#[cfg_attr(
    not(target_os = "windows"),
    ignore = "test only runs on windows platforms."
)]
pub fn test_get_debugger() {
    let debugger_type = DebuggerType::Cdb;

    let result = get_debugger(&debugger_type);
    assert!(result.is_ok());

    let debugger = result.unwrap();
    let re = Regex::new("^cdb: .*cdb.exe$").unwrap();
    assert!(re.is_match(format!("{}", debugger).as_str()));
}

#[test]
pub fn test_debugger_type_from_str() {
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
