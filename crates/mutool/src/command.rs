use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::sync::OnceLock;
use tokio::process::Command as TokioCommand;

static CUSTOM_MUTOOL_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Set a custom path to the `mutool` executable (e.g. from AppDirs or after download).
pub fn set_mutool_path(path: PathBuf) {
  let _ = CUSTOM_MUTOOL_PATH.set(path);
}

/// Retrieve the resolved path to `mutool`.
///
/// Priority:
/// 1. Custom path explicitly set via `set_mutool_path` (if exists).
/// 2. `MUTOOL_PATH` environment variable (if exists).
/// 3. Executable found in system `PATH`.
/// 4. Fallback default executable name (`mutool.exe` on Windows, `mutool` on Unix).
pub fn get_mutool_bin_path() -> PathBuf {
  if let Some(p) = CUSTOM_MUTOOL_PATH.get()
    && p.exists()
  {
    return p.clone();
  }

  if let Ok(env_path) = std::env::var("MUTOOL_PATH") {
    let p = PathBuf::from(env_path);
    if p.exists() {
      return p;
    }
  }

  let exe_name = if cfg!(windows) { "mutool.exe" } else { "mutool" };
  if let Some(paths) = std::env::var_os("PATH") {
    for p in std::env::split_paths(&paths) {
      let candidate = p.join(exe_name);
      if candidate.exists() {
        return candidate;
      }
    }
  }

  PathBuf::from(exe_name)
}

/// Create a new asynchronous `tokio::process::Command` targeting the resolved `mutool` binary.
pub fn mutool_command() -> TokioCommand {
  TokioCommand::new(get_mutool_bin_path())
}

/// Create a new synchronous `std::process::Command` targeting the resolved `mutool` binary.
pub fn mutool_std_command() -> StdCommand {
  StdCommand::new(get_mutool_bin_path())
}
