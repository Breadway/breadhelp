# Shared breadbug consent text

Source of truth: `CONSENT_TEXT` in bread `bread-bug/src/consent.rs` on bread `main`. The terminal reporter shows it before a first session, and `breadbug-gui` and the Bug reports page in bos-settings read the same constant. The text below must match that constant exactly, line for line. A test in `crates/breadhelp-core/src/content/mod.rs` checks that the privacy guide quotes it word for word.

This copy exists because breadhelp cannot depend on bread's Rust crate, so CI has no bread source to read. `scripts/check-consent-sync.py PATH_TO_BREAD_CHECKOUT` compares this file with the constant and is run by hand when the text changes.

Still open: the BOS site does not publish this notice or the privacy statement. That is tracked as a HELP-10 follow-up, not claimed here.

## Notice

Reporting a problem

A report contains your app's recent log lines, crash frames, event names and
basic system facts such as OS, kernel, GPU driver and bread versions. Personal
values such as your name, home folder, hostname, IP addresses, emails and
tokens are replaced before anything is stored or shown. You can read the whole
report before deciding.

Nothing is sent unless you choose Send, and sending is off by default. You can
save the report to a file instead.
