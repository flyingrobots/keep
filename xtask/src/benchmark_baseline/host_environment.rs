//! Bounded capture of operating-system and processor coordinates.

#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::io::Read;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "linux")]
use crate::process_output::bounded_bytes;

use super::BenchmarkBaselineError;
use super::environment::admit_value;
use super::process::{ProcessOutput, run};

const DIAGNOSTIC_LIMIT: usize = 65_536;
const VALUE_LIMIT: usize = 4_096;
#[cfg(target_os = "linux")]
const CPUINFO_READ_LIMIT: u64 = 65_536;

#[derive(Eq, PartialEq)]
pub(super) struct CapturedHost {
    pub(super) os_description: String,
    pub(super) cpu_model: String,
    pub(super) logical_cpu_count: NonZeroUsize,
}

pub(super) fn capture() -> Result<CapturedHost, BenchmarkBaselineError> {
    let logical_cpu_count =
        std::thread::available_parallelism().map_err(|source| BenchmarkBaselineError::Io {
            action: "read logical CPU count from",
            target: Path::new("operating system").to_path_buf(),
            source,
        })?;
    Ok(CapturedHost {
        os_description: os_description()?,
        cpu_model: cpu_model()?,
        logical_cpu_count,
    })
}

#[cfg(unix)]
fn os_description() -> Result<String, BenchmarkBaselineError> {
    coordinate(
        run(
            Command::new("uname").args(["-s", "-r", "-m"]),
            "uname",
            VALUE_LIMIT,
            DIAGNOSTIC_LIMIT,
        )?,
        "os-description",
    )
}

#[cfg(windows)]
fn os_description() -> Result<String, BenchmarkBaselineError> {
    coordinate(
        run(
            Command::new("cmd").args(["/C", "ver"]),
            "cmd",
            VALUE_LIMIT,
            DIAGNOSTIC_LIMIT,
        )?,
        "os-description",
    )
}

#[cfg(not(any(unix, windows)))]
fn os_description() -> Result<String, BenchmarkBaselineError> {
    Err(BenchmarkBaselineError::InvalidValue {
        coordinate: "os-description",
    })
}

#[cfg(target_os = "macos")]
fn cpu_model() -> Result<String, BenchmarkBaselineError> {
    coordinate(
        run(
            Command::new("sysctl").args(["-n", "machdep.cpu.brand_string"]),
            "sysctl",
            VALUE_LIMIT,
            DIAGNOSTIC_LIMIT,
        )?,
        "cpu-model",
    )
}

#[cfg(target_os = "linux")]
fn cpu_model() -> Result<String, BenchmarkBaselineError> {
    let path = Path::new("/proc/cpuinfo");
    let file = File::open(path).map_err(|source| io_error("open", path, source))?;
    let bytes = bounded_bytes(file.take(CPUINFO_READ_LIMIT), DIAGNOSTIC_LIMIT)
        .map_err(|source| io_error("read", path, source))?;
    let contents =
        String::from_utf8(bytes.bytes).map_err(|source| BenchmarkBaselineError::ValueEncoding {
            coordinate: "cpu-model",
            source,
        })?;
    let Some(model) = parse_linux_cpu_model(&contents) else {
        return Err(BenchmarkBaselineError::InvalidValue {
            coordinate: "cpu-model",
        });
    };
    admit_value(String::from(model), "cpu-model")
}

#[cfg(target_os = "linux")]
fn parse_linux_cpu_model(contents: &str) -> Option<&str> {
    ["model name", "Hardware", "Processor"]
        .into_iter()
        .find_map(|wanted| {
            contents.lines().find_map(|line| {
                let (key, value) = line.split_once(':')?;
                (key.trim() == wanted && !value.trim().is_empty()).then_some(value.trim())
            })
        })
}

#[cfg(windows)]
fn cpu_model() -> Result<String, BenchmarkBaselineError> {
    let value = std::env::var("PROCESSOR_IDENTIFIER").map_err(|_source| {
        BenchmarkBaselineError::InvalidValue {
            coordinate: "cpu-model",
        }
    })?;
    admit_value(value, "cpu-model")
}

#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
fn cpu_model() -> Result<String, BenchmarkBaselineError> {
    Err(BenchmarkBaselineError::InvalidValue {
        coordinate: "cpu-model",
    })
}

fn coordinate(output: ProcessOutput, name: &'static str) -> Result<String, BenchmarkBaselineError> {
    if !output.stderr.is_empty() {
        return Err(BenchmarkBaselineError::ReportViolation {
            reason: "successful-environment-command-wrote-diagnostics",
        });
    }
    let value = String::from_utf8(output.stdout).map_err(|source| {
        BenchmarkBaselineError::ValueEncoding {
            coordinate: name,
            source,
        }
    })?;
    let value = value
        .strip_suffix("\r\n")
        .or_else(|| value.strip_suffix('\n'))
        .unwrap_or(&value);
    admit_value(String::from(value), name)
}

#[cfg(target_os = "linux")]
fn io_error(action: &'static str, target: &Path, source: std::io::Error) -> BenchmarkBaselineError {
    BenchmarkBaselineError::Io {
        action,
        target: target.to_path_buf(),
        source,
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::parse_linux_cpu_model;

    #[test]
    fn linux_cpu_model_prefers_specific_model_name_over_generic_processor() {
        let contents = "Processor : generic\nHardware : board\nmodel name : precise model\n";

        assert_eq!(parse_linux_cpu_model(contents), Some("precise model"));
    }
}
