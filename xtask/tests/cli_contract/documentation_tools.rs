//! This module owns hermetic fake documentation tools for CLI verification.

#![allow(
    clippy::redundant_pub_crate,
    reason = "the parent integration-test module owns this private fixture"
)]

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct DocumentationTools {
    root: Option<PathBuf>,
}

impl DocumentationTools {
    pub(crate) fn create() -> Result<Self, io::Error> {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| io::Error::other("xtask manifest has no repository parent"))?;
        let parent = repository_root.join("target/cli-contract-tools");
        fs::create_dir_all(&parent)?;
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = parent.join(format!("{}-{sequence}", std::process::id()));
        fs::create_dir(&root)?;
        let tools = Self { root: Some(root) };
        fs::create_dir(tools.bin()?)?;
        fs::create_dir(tools.markers()?)?;
        tools.install(
            "markdownlint-cli2",
            "--version",
            "markdownlint-cli2 v0.23.2 (markdownlint v0.41.1)",
        )?;
        tools.install("lychee", "--version", "lychee 0.21.0")?;
        tools.install("actionlint", "-version", "1.7.12")?;
        Ok(tools)
    }

    pub(crate) fn invoke(&self, arguments: &[&str]) -> Result<Output, io::Error> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(arguments)
            .env("PATH", self.path_environment()?)
            .output()
    }

    pub(crate) fn require_every_tool(&self) -> Result<(), io::Error> {
        for program in ["markdownlint-cli2", "lychee", "actionlint"] {
            let marker = self.markers()?.join(program);
            if !marker.is_file() {
                return Err(io::Error::other(format!(
                    "documentation tool was not invoked: {program}"
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn close(mut self) -> Result<(), io::Error> {
        let root = self
            .root
            .take()
            .ok_or_else(|| io::Error::other("documentation tool directory is already closed"))?;
        fs::remove_dir_all(root)
    }

    fn install(
        &self,
        program: &str,
        version_argument: &str,
        version: &str,
    ) -> Result<(), io::Error> {
        let marker = shell_word(&self.markers()?.join(program))?;
        let script = format!(
            "#!/bin/sh\n\
             : > {marker}\n\
             for argument in \"$@\"; do\n\
             \x20 if [ \"$argument\" = \"{version_argument}\" ]; then\n\
             \x20   printf '%s\\n' '{version}'\n\
             \x20   exit 0\n\
             \x20 fi\n\
             done\n\
             exit 0\n"
        );
        let path = self.bin()?.join(program);
        fs::write(&path, script)?;
        let mut permissions = fs::metadata(&path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)
    }

    fn path_environment(&self) -> Result<OsString, io::Error> {
        let existing = env::var_os("PATH").unwrap_or_default();
        env::join_paths(std::iter::once(self.bin()?).chain(env::split_paths(&existing)))
            .map_err(io::Error::other)
    }

    fn bin(&self) -> Result<PathBuf, io::Error> {
        Ok(self.root()?.join("bin"))
    }

    fn markers(&self) -> Result<PathBuf, io::Error> {
        Ok(self.root()?.join("markers"))
    }

    fn root(&self) -> Result<&Path, io::Error> {
        self.root
            .as_deref()
            .ok_or_else(|| io::Error::other("documentation tool directory is closed"))
    }
}

fn shell_word(path: &Path) -> Result<String, io::Error> {
    let text = path
        .to_str()
        .ok_or_else(|| io::Error::other("documentation tool path is not UTF-8"))?;
    Ok(format!("'{}'", text.replace('\'', "'\\''")))
}

impl Drop for DocumentationTools {
    fn drop(&mut self) {
        if let Some(root) = self.root.take() {
            drop(fs::remove_dir_all(root));
        }
    }
}
