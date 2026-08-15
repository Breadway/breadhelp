use std::path::Path;
use std::process::Command;

/// Fire-and-forget: runs `command` via a shell, matching the idiom already
/// used throughout bos-settings' views (`let _ = Command::new(...).spawn()`).
pub fn run(command: &str) {
    if let Err(e) = Command::new("sh").arg("-c").arg(command).spawn() {
        eprintln!("breadhelp: failed to run `{command}`: {e}");
    }
}

/// Fire-and-forget argv form — use this when arguments come from the UI
/// (timezone names, page ids) so they never pass through a shell.
pub fn run_argv(prog: &str, args: &[&str]) {
    if let Err(e) = Command::new(prog).args(args).spawn() {
        eprintln!("breadhelp: failed to run `{prog}`: {e}");
    }
}

/// `true` when `name` resolves to an executable on `PATH`.
pub fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(name))))
        .unwrap_or(false)
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

/// Runs `command` on a background thread and delivers success/failure back
/// onto the GTK main loop via `on_done` — plain `spawn()` only confirms the
/// process *launched*, not how it exited, and one-click fixes need to report
/// that back to the user (a toast), so this blocks a throwaway thread on
/// `Command::status()` instead of the UI thread.
pub fn run_reporting(command: &str, on_done: impl Fn(bool) + 'static) {
    let command = command.to_string();
    let (tx, rx) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let status = Command::new("sh").arg("-c").arg(&command).status();
        let _ = tx.send_blocking(status.map(|s| s.success()).unwrap_or(false));
    });
    glib::MainContext::default().spawn_local(async move {
        if let Ok(success) = rx.recv().await {
            on_done(success);
        }
    });
}

/// Same as [`run_reporting`], but without a shell.
pub fn run_argv_reporting(prog: &str, args: &[&str], on_done: impl Fn(bool) + 'static) {
    let prog = prog.to_string();
    let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
    let (tx, rx) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let status = Command::new(&prog).args(&args).status();
        let _ = tx.send_blocking(status.map(|s| s.success()).unwrap_or(false));
    });
    glib::MainContext::default().spawn_local(async move {
        if let Ok(success) = rx.recv().await {
            on_done(success);
        }
    });
}
