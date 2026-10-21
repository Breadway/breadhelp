use std::path::Path;
use std::process::Command;

/// Spawn without waiting, but reap the child on a throwaway thread: the help
/// window stays resident for the whole session, so an un-waited child would
/// sit as a zombie until logout.
pub fn spawn_detached(mut cmd: Command) -> std::io::Result<()> {
    let mut child = cmd.spawn()?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Fire-and-forget: runs `command` via a shell.
pub fn run(command: &str) {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);
    if let Err(e) = spawn_detached(cmd) {
        eprintln!("breadhelp: failed to run `{command}`: {e}");
    }
}

/// Fire-and-forget argv form: use this when arguments come from the UI
/// (timezone names, page ids) so they never pass through a shell.
pub fn run_argv(prog: &str, args: &[&str]) {
    let mut cmd = Command::new(prog);
    cmd.args(args);
    if let Err(e) = spawn_detached(cmd) {
        eprintln!("breadhelp: failed to run `{prog}`: {e}");
    }
}

/// Split a content-authored terminal command on whitespace into argv.
/// No shell is involved downstream, so shell metacharacters in guide text
/// can never be interpreted.
pub fn split_argv(command: &str) -> Vec<String> {
    command.split_whitespace().map(str::to_string).collect()
}

/// Open the BOS terminal (kitty, the same terminal bos-settings launches
/// for terminal work) running `argv`, without a shell. Used for
/// terminal-only commands such as `breadbug`, which need a TTY and fail
/// when spawned bare from this GUI (its desktop entry is
/// `Terminal=false`).
pub fn run_terminal(argv: &[String]) {
    if argv.is_empty() {
        return;
    }
    let mut cmd = Command::new("kitty");
    cmd.arg("-e");
    cmd.args(argv);
    if let Err(e) = spawn_detached(cmd) {
        eprintln!("breadhelp: failed to open terminal for `{argv:?}`: {e}");
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

/// Runs `command` through a shell and waits for it: plain `spawn()` only
/// confirms the process *launched*, not how it exited, and one-click fixes
/// report that back to the user. Blocks, so callers run it off their UI
/// thread (the help window calls it from an async Tauri command).
pub fn run_status(command: &str) -> bool {
    Command::new("sh").arg("-c").arg(command).status().map(|s| s.success()).unwrap_or(false)
}

/// Same as [`run_status`], but without a shell.
pub fn run_argv_status(prog: &str, args: &[&str]) -> bool {
    Command::new(prog).args(args).status().map(|s| s.success()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_commands_split_on_whitespace_only() {
        assert_eq!(split_argv("breadbug"), vec!["breadbug".to_string()]);
        assert_eq!(
            split_argv("breadbug --app clip --save report.json"),
            vec!["breadbug", "--app", "clip", "--save", "report.json"]
        );
    }

    #[test]
    fn shell_metacharacters_stay_literal_words() {
        // These must reach the terminal as literal words, never interpreted:
        // `run_terminal` passes argv straight to `kitty -e` with no shell.
        assert_eq!(
            split_argv("breadbug --notes $(evil) `evil` a;b | c"),
            vec!["breadbug", "--notes", "$(evil)", "`evil`", "a;b", "|", "c"]
        );
    }

    #[test]
    fn blank_terminal_command_is_empty() {
        assert!(split_argv("   ").is_empty());
    }
}
