//! Integration laws for descriptor-bound child working directories.

use std::env;
use std::fs::{self, File};
use std::io;
use std::os::fd::OwnedFd;
use std::path::PathBuf;
use std::process::{self, Command};

use repository_process_spawn::set_working_directory;

#[test]
fn child_uses_the_opened_directory_without_mutating_parent_state()
-> Result<(), Box<dyn std::error::Error>> {
    let parent = env::current_dir()?;
    let world = test_world("replacement");
    let root = world.join("repository");
    let retained = world.join("retained");
    fs::create_dir_all(&root)?;
    fs::write(root.join("marker"), b"original\n")?;
    let directory: OwnedFd = File::open(&root)?.into();

    fs::rename(&root, &retained)?;
    fs::create_dir(&root)?;
    fs::write(root.join("marker"), b"substitute\n")?;

    let mut command = Command::new("cat");
    command.arg("marker");
    set_working_directory(&mut command, directory);
    let output = command.output()?;

    fs::remove_dir_all(&root)?;
    fs::remove_dir_all(&retained)?;
    fs::remove_dir(&world)?;

    assert!(output.status.success());
    assert_eq!(output.stdout, b"original\n");
    assert_eq!(env::current_dir()?, parent);
    Ok(())
}

#[test]
fn non_directory_descriptor_refuses_before_exec() -> Result<(), Box<dyn std::error::Error>> {
    let world = test_world("non-directory");
    fs::create_dir(&world)?;
    let file_path = world.join("file");
    fs::write(&file_path, b"not a directory\n")?;
    let descriptor: OwnedFd = File::open(&file_path)?.into();
    let mut command = Command::new("true");
    set_working_directory(&mut command, descriptor);

    let result = command.spawn();
    fs::remove_file(&file_path)?;
    fs::remove_dir(&world)?;
    let error = match result {
        Ok(mut child) => {
            let termination = child.kill();
            let reap = child.wait();
            return Err(format!(
                "non-directory descriptor unexpectedly reached exec; termination: {termination:?}; reap: {reap:?}"
            )
            .into());
        }
        Err(error) => error,
    };

    assert_eq!(error.kind(), io::ErrorKind::NotADirectory);
    Ok(())
}

fn test_world(case: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "keep-repository-process-spawn-{}-{case}",
        process::id()
    ))
}
