# Shared breadbug consent text

Source of truth: `CONSENT_TEXT` in bread `bread-bug/src/consent.rs` (bread `main`, merged in PR 31). The reporter shows it before a first session. breadbug-gui reads the same constant. The text below must match that constant exactly, line for line. A test in `src/content/mod.rs` checks that the privacy guide quotes it word for word.

Cross-repo gates, not done: bos-settings does not yet read this text (no Bug reports page exists), and the BOS site does not publish it. Until a single source is read by both, this copy is the breadhelp copy only.

## Notice

Reporting a problem

A report contains your app's recent log lines, crash frames, event names and
basic system facts such as OS, kernel, GPU driver and bread versions. Personal
values such as your name, home folder, hostname, IP addresses, emails and
tokens are replaced before anything is stored or shown. You can read the whole
report before deciding.

Nothing is sent unless you choose Send, and sending is off by default. You can
save the report to a file instead.
