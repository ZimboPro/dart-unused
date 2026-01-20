use std::path::PathBuf;

/// Sets the current working directory to the given path.
pub fn set_current_dir(path: &PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        panic!("Path does not exist");
    } else if path.is_dir() {
        std::env::set_current_dir(path)?;
    } else {
        let parent = path.parent().expect("Failed to get parent directory");
        std::env::set_current_dir(parent)?;
    }
    Ok(())
}

/// Gets the system path to the `dart` command.
pub fn get_dart_command_path() -> anyhow::Result<String> {
    #[cfg(windows)]
    let c = "where";
    #[cfg(unix)]
    let c = "which";
    let s = std::process::Command::new(c).args(["dart"]).output()?;
    let s = String::from_utf8(s.stdout)?;
    let l = s.lines();
    let l: Vec<&str> = l.into_iter().filter(|x| !x.is_empty()).collect();
    if l.is_empty() {
        return Err(anyhow::anyhow!("Could not find dart command"));
    }
    Ok(l.last().expect("Failed to get dart command").to_string())
}
