# Shared breadbug consent text (single source inside breadhelp)

Provenance: the wording below is copied from bread `bread-bug/src/consent.rs`
(`CONSENT_TEXT`) on bread `main`, where the consent notice is merged and shown by
the terminal reporter and the GUI. It must stay identical to that constant.

Cross-repo prerequisites (tracked, not claimed): a Bug reports page in
bos-settings that reads this text, and publication on the BOS site. Neither
exists yet, so this breadhelp copy is still the only place the notice is kept
apart from the code.

## Notice

Reporting a problem

A report contains your app's recent log lines, crash frames, event names and
basic system facts such as OS, kernel, GPU driver and bread versions. Personal
values such as your name, home folder, hostname, IP addresses, emails and
tokens are replaced before anything is stored or shown. You can read the whole
report before deciding.

Nothing is sent unless you choose Send, and sending is off by default. You can
save the report to a file instead.
