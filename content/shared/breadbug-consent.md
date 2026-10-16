# Shared breadbug consent text (single source inside breadhelp)

Provenance: the proposed wording below is taken from bread
`feature/breadbug-consent` (`bread-bug/src/consent.rs`, CONSENT_TEXT) at
1e49507a6c506fa1023bf3c201c574ed5abc763f. That branch is NOT merged, so no
shipped reporter or settings page shows this text yet. HELP-10 requires this
statement to become the same text the reporter shows and the settings page
shows; until that branch lands, the privacy guide quotes the proposal from
this file, and both guides point here instead of each carrying their own copy.

Cross-repo prerequisites (tracked, not claimed): bread
`feature/breadbug-consent` merged, a Bug reports page in bos-settings, and
publication on the BOS site. None of the three exists yet.

## Proposed notice

Reporting a problem

A report contains your app's recent log lines, crash frames, event names and
basic system facts such as OS, kernel, GPU driver and bread versions. Personal
values such as your name, home folder, hostname, IP addresses, emails and
tokens are replaced before anything is stored or shown. You can read the whole
report before deciding.

Nothing is sent unless you choose Send, and sending is off by default. You can
save the report to a file instead.
