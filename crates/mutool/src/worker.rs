use crate::PageLink;
use crate::mutool_error::MuToolError;
use crate::page_info::{DocumentPageInfo, PageDimensions};
use crate::stext::PageStructuredText;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock};

const WORKER_SCRIPT_SRC: &str = include_str!("scripts/worker.js");
static SCRIPT_PATH_CACHE: OnceLock<PathBuf> = OnceLock::new();
static GLOBAL_WORKER_POOL: LazyLock<MutoolWorkerPool> = LazyLock::new(MutoolWorkerPool::new);

pub fn ensure_worker_script() -> Result<PathBuf, MuToolError> {
  if let Some(path) = SCRIPT_PATH_CACHE.get()
    && path.is_file()
  {
    return Ok(path.clone());
  }

  let tmp_path = std::env::temp_dir().join("libera_mupdf_worker_v1.js");
  // Check if file exists and has correct content length or write it
  let need_write = match std::fs::read_to_string(&tmp_path) {
    Ok(content) => content != WORKER_SCRIPT_SRC,
    Err(_) => true,
  };

  if need_write {
    std::fs::write(&tmp_path, WORKER_SCRIPT_SRC).map_err(|_| MuToolError::IoError)?;
  }

  let _ = SCRIPT_PATH_CACHE.set(tmp_path.clone());
  Ok(tmp_path)
}

#[derive(Serialize, Debug)]
struct WorkerRequest<'a> {
  id: u64,
  cmd: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  path: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  page: Option<usize>,
}

#[derive(Deserialize, Debug)]
struct WorkerResponse<T> {
  #[allow(dead_code)]
  id: Option<u64>,
  result: Option<T>,
  error: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OpenDocResult {
  #[allow(dead_code)]
  page_count: usize,
}

pub struct MutoolWorkerSession {
  process: Child,
  stdin: ChildStdin,
  reader: BufReader<ChildStdout>,
  next_id: AtomicU64,
  current_path: Option<PathBuf>,
  pub healthy: bool,
}

impl MutoolWorkerSession {
  pub fn start() -> Result<Self, MuToolError> {
    let script_path = ensure_worker_script()?;
    let mutool_bin = crate::get_mutool_bin_path();

    // On Unix, use stdbuf to force line-buffering on stdout and unbuffered stdin.
    // Otherwise glibc defaults to block buffering (4KB+) when stdout is a pipe,
    // which causes deadlock on read_line().
    let mut process = if cfg!(unix) {
      let mut cmd = std::process::Command::new("stdbuf");
      cmd
        .arg("-i0")
        .arg("-oL")
        .arg(&mutool_bin)
        .arg("run")
        .arg(&script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
      match cmd.spawn() {
        Ok(p) => p,
        Err(_) => {
          // Fallback if stdbuf is not found in PATH
          crate::mutool_std_command()
            .arg("run")
            .arg(&script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| MuToolError::MutoolNotFound)?
        }
      }
    } else {
      crate::mutool_std_command()
        .arg("run")
        .arg(&script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| MuToolError::MutoolNotFound)?
    };

    let stdin = process.stdin.take().ok_or(MuToolError::IoError)?;
    let stdout = process.stdout.take().ok_or(MuToolError::IoError)?;
    let reader = BufReader::new(stdout);

    let mut session = Self {
      process,
      stdin,
      reader,
      next_id: AtomicU64::new(1),
      current_path: None,
      healthy: true,
    };
    // Verify the worker is live and responsive with a quick ping
    if !session.ping().unwrap_or(false) {
      return Err(MuToolError::OtherErr);
    }

    Ok(session)
  }

  pub fn call<T: for<'de> Deserialize<'de>>(
    &mut self, cmd: &str, path: Option<&Path>, page: Option<usize>,
  ) -> Result<T, MuToolError> {
    self.healthy = false;
    let id = self.next_id.fetch_add(1, Ordering::SeqCst);
    let path_str = path.and_then(|p| p.to_str());
    let req = WorkerRequest { id, cmd, path: path_str, page };

    let mut line =
      serde_json::to_string(&req).map_err(|e| MuToolError::JsonError(e.to_string()))?;
    line.push('\n');

    self.stdin.write_all(line.as_bytes()).map_err(|_| MuToolError::IoError)?;
    self.stdin.flush().map_err(|_| MuToolError::IoError)?;

    let mut response_line = String::new();
    let bytes_read = self.reader.read_line(&mut response_line).map_err(|_| MuToolError::IoError)?;

    if bytes_read == 0 {
      return Err(MuToolError::OtherErr);
    }

    let resp: WorkerResponse<T> = serde_json::from_str(&response_line)
      .map_err(|e| MuToolError::JsonError(format!("{e}: {response_line}")))?;

    if let Some(_err) = resp.error {
      return Err(MuToolError::OtherErr);
    }

    let res = resp.result.ok_or(MuToolError::OtherErr)?;
    self.healthy = true;
    Ok(res)
  }

  pub fn ping(&mut self) -> Result<bool, MuToolError> {
    let res: String = self.call("ping", None, None)?;
    Ok(res == "pong")
  }

  pub fn open(&mut self, path: &Path) -> Result<usize, MuToolError> {
    let res: OpenDocResult = self.call("open", Some(path), None)?;
    self.current_path = Some(path.to_path_buf());
    Ok(res.page_count)
  }

  pub fn close(&mut self) -> Result<(), MuToolError> {
    let _: bool = self.call("close", None, None)?;
    self.current_path = None;
    Ok(())
  }

  pub fn get_page_links(&mut self, path: &Path, page: usize) -> Result<Vec<PageLink>, MuToolError> {
    self.call("get_page_links", Some(path), Some(page))
  }

  pub fn get_page_info(&mut self, path: &Path) -> Result<DocumentPageInfo, MuToolError> {
    self.call("get_page_info", Some(path), None)
  }

  pub fn get_page_size(&mut self, path: &Path, page: usize) -> Result<PageDimensions, MuToolError> {
    self.call("get_page_size", Some(path), Some(page))
  }

  pub fn get_page_structured_text(
    &mut self, path: &Path, page: usize,
  ) -> Result<PageStructuredText, MuToolError> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum STextResponse {
      Wrapped { pages: Vec<PageStructuredText> },
      Direct(PageStructuredText),
    }

    let resp: STextResponse = self.call("get_stext", Some(path), Some(page))?;
    match resp {
      STextResponse::Direct(stext) => Ok(stext),
      STextResponse::Wrapped { mut pages } => Ok(pages.drain(..).next().unwrap_or_default()),
    }
  }
}

impl Drop for MutoolWorkerSession {
  fn drop(&mut self) {
    let _ = self.stdin.write_all(b"{\"cmd\":\"close\"}\n");
    let _ = self.stdin.flush();
    let _ = self.process.kill();
    let _ = self.process.wait();
  }
}

pub struct MutoolWorkerPool {
  sessions: Mutex<Vec<MutoolWorkerSession>>,
}

impl Default for MutoolWorkerPool {
  fn default() -> Self {
    Self::new()
  }
}

impl MutoolWorkerPool {
  pub fn new() -> Self {
    Self { sessions: Mutex::new(Vec::new()) }
  }

  pub fn acquire(&self) -> Result<PooledWorker<'_>, MuToolError> {
    let mut lock = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(session) = lock.pop() {
      Ok(PooledWorker { session: Some(session), pool: self })
    } else {
      let session = MutoolWorkerSession::start()?;
      Ok(PooledWorker { session: Some(session), pool: self })
    }
  }

  fn release(&self, session: MutoolWorkerSession) {
    let mut lock = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
    if lock.len() < 4 {
      lock.push(session);
    }
  }
}

pub struct PooledWorker<'a> {
  session: Option<MutoolWorkerSession>,
  pool: &'a MutoolWorkerPool,
}

impl<'a> std::ops::Deref for PooledWorker<'a> {
  type Target = MutoolWorkerSession;
  fn deref(&self) -> &Self::Target {
    self.session.as_ref().unwrap()
  }
}

impl<'a> std::ops::DerefMut for PooledWorker<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.session.as_mut().unwrap()
  }
}

impl<'a> Drop for PooledWorker<'a> {
  fn drop(&mut self) {
    if let Some(session) = self.session.take()
      && session.healthy
    {
      self.pool.release(session);
    }
  }
}

pub fn global_worker_pool() -> &'static MutoolWorkerPool {
  &GLOBAL_WORKER_POOL
}
