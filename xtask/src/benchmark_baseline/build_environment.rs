//! Admission of host build inputs that can change benchmark machine code.

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::BenchmarkBaselineError;

const CONFIG_NAMES: [&str; 2] = ["config", "config.toml"];
const EXACT_SETTINGS: [&str; 18] = [
    "AR",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_INCREMENTAL",
    "CC",
    "CFLAGS",
    "CRATE_CC_NO_DEFAULTS",
    "CXX",
    "CXXFLAGS",
    "DEVELOPER_DIR",
    "MACOSX_DEPLOYMENT_TARGET",
    "RANLIB",
    "RUSTC",
    "RUSTC_BOOTSTRAP",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTC_WRAPPER",
    "RUSTDOCFLAGS",
    "RUSTFLAGS",
    "SDKROOT",
];
const SETTING_PREFIXES: [&str; 10] = [
    "AR_",
    "CARGO_BUILD_",
    "CARGO_PROFILE_",
    "CARGO_TARGET_",
    "CC_",
    "CFLAGS_",
    "CXXFLAGS_",
    "CXX_",
    "HOST_",
    "TARGET_",
];

pub(super) fn admit(repository_root: &Path) -> Result<(), BenchmarkBaselineError> {
    admit_variables(env::vars_os().map(|(name, _value)| name))?;
    admit_external_config(repository_root, cargo_home())
}

fn admit_variables(
    names: impl IntoIterator<Item = OsString>,
) -> Result<(), BenchmarkBaselineError> {
    let settings = names
        .into_iter()
        .filter_map(|name| name.into_string().ok())
        .filter(|name| affects_machine_code(name))
        .collect::<BTreeSet<_>>();
    settings.into_iter().next().map_or(Ok(()), |setting| {
        Err(BenchmarkBaselineError::AmbientBuildSetting { setting })
    })
}

fn affects_machine_code(name: &str) -> bool {
    EXACT_SETTINGS.contains(&name)
        || SETTING_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
}

fn admit_external_config(
    repository_root: &Path,
    cargo_home: Option<PathBuf>,
) -> Result<(), BenchmarkBaselineError> {
    let mut candidates = BTreeSet::new();
    for ancestor in repository_root.ancestors().skip(1) {
        add_config_candidates(&mut candidates, &ancestor.join(".cargo"));
    }
    if let Some(cargo_home) = cargo_home {
        add_config_candidates(&mut candidates, &cargo_home);
    }
    for path in candidates {
        match fs::symlink_metadata(&path) {
            Ok(_metadata) => {
                return Err(BenchmarkBaselineError::ExternalCargoConfiguration { path });
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(BenchmarkBaselineError::Io {
                    action: "inspect external Cargo configuration",
                    target: path,
                    source,
                });
            }
        }
    }
    Ok(())
}

fn add_config_candidates(candidates: &mut BTreeSet<PathBuf>, directory: &Path) {
    for name in CONFIG_NAMES {
        candidates.insert(directory.join(name));
    }
}

fn cargo_home() -> Option<PathBuf> {
    env::var_os("CARGO_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .map(|home| home.join(".cargo"))
        })
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::ffi::OsString;
    use std::fs;

    use super::{admit_external_config, admit_variables, affects_machine_code};
    use crate::benchmark_baseline::BenchmarkBaselineError;
    use crate::test_directory::TestDirectory;

    #[test]
    fn code_generation_setting_catalog_covers_cargo_rustc_and_linkers() {
        for name in [
            "CARGO_PROFILE_RELEASE_OPT_LEVEL",
            "CARGO_BUILD_TARGET",
            "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS",
            "RUSTFLAGS",
            "RUSTC_WRAPPER",
            "CC_AARCH64_APPLE_DARWIN",
            "MACOSX_DEPLOYMENT_TARGET",
        ] {
            assert!(affects_machine_code(name), "{name}");
        }
        assert!(!affects_machine_code("CARGO_TERM_COLOR"));
        assert!(admit_variables([OsString::from("CARGO_TERM_COLOR")]).is_ok());
    }

    #[test]
    fn only_source_bound_cargo_configuration_can_shape_baselines() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::create("benchmark-cargo-config")?;
        let repository = directory.path().join("workspace/repository");
        let source_config = repository.join(".cargo/config.toml");
        let external_config = directory.path().join("workspace/.cargo/config.toml");
        fs::create_dir_all(
            source_config
                .parent()
                .ok_or("source config has no parent")?,
        )?;
        fs::create_dir_all(
            external_config
                .parent()
                .ok_or("external config has no parent")?,
        )?;
        fs::write(&source_config, "[alias]\n")?;

        assert!(admit_external_config(&repository, None).is_ok());

        fs::write(
            &external_config,
            "[build]\nrustflags = [\"-C\", \"opt-level=0\"]\n",
        )?;
        assert!(matches!(
            admit_external_config(&repository, None),
            Err(BenchmarkBaselineError::ExternalCargoConfiguration { path })
                if path == external_config
        ));
        directory.close()?;
        Ok(())
    }
}
