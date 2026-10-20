# Bug reports and your privacy

This page says what breadbug collects, what it removes, what it never touches, and what happens when you send a report. Every statement describes the code on bread `main` (the merged breadbug client) or the breadbug-server code on its `main`. Anything that is not in place yet is listed under Not yet shipped and is not promised.

## What a report contains

Schema version 1. These are the top-level sections. `breadbug --app ID --dry-run` prints the exact report, so you can check it.

- **App**: the app id, its installed version and its build id, when known.
- **System facts**: OS id and version, OS build, kernel, CPU architecture, GPU driver with PCI vendor and device ids, compositor version, bakery track, and breadd and API versions.
- **Installed bread apps**: app ids and versions for the bread apps bakery reports as installed.
- **Runtime**: whether the daemon is running, its uptime, adapter status strings, and module counts.
- **Markers**: up to the 5 most recent panic, crash, freeze and recovery records. Each has a time, an optional message and location, and up to 128 stack frames.
- **Logs**: recent log lines from the app, and the daemon's warnings and errors.
- **Events**: bread event names. Only these fields survive: battery percentage and AC state, network online state, and device subsystem, vendor and product ids. Other fields are dropped, and only the event name is kept.
- **Journal**: recent `journalctl --user` lines from the app's own processes. Each line is cut at 2048 bytes.
- **Live stack**: stack frames of a frozen app that is still running. Only present if you agreed to that capture for this report.
- **Notes**: the note you typed, scrubbed, at most 8 KiB.
- **Redactions**: how many values the scrubber replaced in each category.

A report is capped at 1 MiB. If it is too large, the builder drops journal lines first, then the oldest events and logs.

## What is replaced before anything is stored or shown

The scrubber replaces these with markers such as `<HOME>`, `<IP:1>` or `<TOKEN>`:

- Your user name and computer name (`<USER>`, `<HOST>`) and your home folder (`<HOME>`).
- IP addresses (`<IP>`), MAC addresses (`<MAC>`), email addresses (`<EMAIL>`) and device serial numbers (`<SERIAL>`).
- Wi-Fi network names (`<SSID>`) and window titles (`<TITLE>`) where they appear as `title=` or `ssid=` style values in text.
- Passwords, secrets and private keys (`<SECRET>`), recognised token formats such as GitHub, GitLab, Slack, AWS and JWT tokens (`<TOKEN>`), long random-looking strings (`<TOKEN>` or `<HEX>`), credentials in web addresses (`<CRED>`), and query strings (`<QUERY>`).
- Build ids are exempt from the hex and random-string rules, because symbol lookup needs them.

Scrubbing runs twice: when the daemon stores a log line, and again over the whole report with a fresh scrubber before you preview it. The preview shows how many values were replaced in each category.

Limits you should know:

- Scrubbing is best effort, and it works on text patterns. It cannot know that an ordinary word is a person's name. Read the preview.
- Your notes are scrubbed like everything else, but they are free text. Do not paste passwords or private text into them.
- Event fields are allowlisted, so event payloads outside the list are dropped. Free text in logs, markers and the journal is only pattern-scrubbed.
- The journal is read directly from `journalctl` when the report is built, not from breadbug's store. Excluding an app (see below) does not remove its journal lines from a report. Check the preview.

## What is never collected

These are the things the code does not read or send:

- **Screenshots.** No code in breadbug or breadd captures the screen.
- **Process memory, registers and environment variables.** A live stack is taken with `eu-stack`, which reports module names, module-relative offsets and build ids. It never reads memory.
- **Core dumps.**

These are not collected by the report schema above. They are not guaranteed to be absent from every log line an app writes, so do not rely on this list alone and read the preview:

- **Memory contents** of apps, other than what an app itself writes to its logs.
- **Window titles and personal files.** No report section is built from window titles or from the contents of files in your home folder. A window title or file path can still appear in a log line or the journal if an app printed it there, and the scrubber only removes it when it matches a pattern.

Breadbug has no default exclusion for apps that handle clipboard contents or search queries. Exclusion is opt-in in `breadbug.toml`. The code has no list of sensitive apps yet.

## Sending is off by default

Sending needs two things: `upload_enabled = true` (set with `breadbug config upload on`), and a separate explicit Send for that one report. The setting is off by default. While it is off, breadbug does not contact the network. That includes the queue, `history --refresh`, `status` and `delete`.

Saving a report to a file never needs the network, and it is always allowed.

## Where a sent report goes

- Reports go over HTTPS to the endpoint in `breadbug.toml`. The default is `https://bugs.breadway.dev`. Plain HTTP is accepted only for a loopback address. Redirects are never followed.
- The exact previewed bytes are sent unchanged. The client checks their hash right before sending. The server checks the `X-Breadbug-Sha256` hash too.
- The server scrubs every report again with the same rules. A report with anything new to redact is quarantined.

## What the server keeps, and for how long

This section describes the server code in the breadbug-server repository, not anything breadbug on your machine enforces.

- **Raw reports** are deleted once they are older than the retention period. The default is 90 days (`BREADBUG_RETENTION_DAYS`). A purge runs every hour (`src/app.rs`, `maintain`; `src/config.rs`, `retention`).
- **Cluster metadata** is kept after raw reports are deleted. This is grouping data: a kind, an app, counts, versions and OS builds, and first and last seen times. It is not the raw report (`docs/threat-model.md`).
- **Install ids** are stored only as a keyed hash (`src/app.rs`, `install_hash`). The server does not store the install id itself.
- **Reports are also re-scrubbed on the server.** A report with anything new to redact is quarantined and kept out of clustering (`src/app.rs`, `store_report`).
- **IP addresses** are not stored. The server's network bucket is a keyed hash held only in memory for rate limiting, and it is never persisted (`src/app.rs`, `network_key`; `src/ratelimit.rs`). The reports table has no address column (`src/db.rs`).
- **Logs**: the threat model says report content is never written to logs (`docs/threat-model.md`).

Not guaranteed by the code in this repository:

- Whether the reverse proxy in front of the server keeps access logs. The threat model says access logging is disabled for the host, but that setting is in the infra repository, which is not checked here.
- What happens to a report in a backup. Backups are kept by the operator. A deleted report stays in a backup until that backup is rotated (`README.md`, backup and deletion contract). Copies made outside the tool's control are not erased by it.

## Deleting a report you sent

When a send succeeds, the server returns a receipt and a deletion token. breadbug keeps both in `history.jsonl`, readable only by you.

- `breadbug history` lists the reports you sent or saved, with their receipts.
- `breadbug delete <receipt>` asks the server to delete that report, using the deletion token kept on this machine. On success, breadbug removes the entry from your history. It needs sending to be on.
- A report the server recognised as an identical earlier send returns no new token. That report has no token on your machine, so `breadbug delete` cannot delete it from here.

What a server-side delete removes: the raw report row is deleted, and its contribution to the cluster counts is taken back out. The database is then compacted (`src/db.rs`, `delete_report` and `compact`; `src/app.rs`, `delete_report`).

What it does not remove:

- **Copies in backups**, until the backup is rotated.
- **Breadboard issues**. When a cluster is promoted to a breadboard issue, the issue body includes up to three evidence samples from its reports. Each sample is built from the report's title, its `want` text, its last marker message and up to 8 frames, and its live stack frames (`src/lifecycle.rs`, `issue_body` and `evidence`). Breadbug's client does not set `title` or `want`, so for reports sent by breadbug the sample is the marker message and frames. Deleting a raw report does not edit or delete that issue. This is not yet implemented.

To ask for a report to be removed when `breadbug delete` is not available to you, there is no request path yet. See Not yet shipped.

## Install id

- The install id is 16 random bytes, written as 32 hex characters. It is created on first use and stored only on this machine, in `~/.local/state/bread/breadbug/install.json`.
- It is replaced when it is older than 30 days (`INSTALL_ID_LIFETIME_SECS` in `bread-bug/src/upload.rs`).
- There is no account and no device identifier. The app does not read a hardware serial.

## Queue and receipts

- If the server cannot be reached, or returns a server error or 429, the exact report is saved to an offline queue of up to 20 reports (`MAX_QUEUED`). The queue retries with increasing waits, capped at 6 hours (`MAX_BACKOFF_SECS`).
- Retries happen when you run `breadbug queue flush`. The daily timer runs `breadbug status`, which only checks receipts, so it does not send queued reports (`packaging/systemd/breadbug-status.service`, `breadbug/src/lib.rs`, `queue`).
- A report the server rejects is dropped and not tried again.
- Use `breadbug queue` to see what is waiting, and `breadbug queue cancel <id>` or `breadbug queue clear` to drop reports.

## Your controls

- Save a report to a file instead of sending it.
- Turn sending on or off with `breadbug config upload on` or `off`.
- Exclude an app in `breadbug.toml` so the daemon stores nothing for it. The journal caveat above still applies.
- Read your history with `breadbug history`.
- Check on your sent reports with `breadbug status`. It asks the server only about receipts you hold, and it needs sending to be on.
- Delete a report you sent with `breadbug delete <receipt>`.

## Consent notice

This is the notice breadbug shows before your first interactive session. It is the same text as the `CONSENT_TEXT` constant in `bread-bug/src/consent.rs`, which is the source. A test in breadhelp checks that this page quotes it word for word.

Reporting a problem

A report contains your app's recent log lines, crash frames, event names and
basic system facts such as OS, kernel, GPU driver and bread versions. Personal
values such as your name, home folder, hostname, IP addresses, emails and
tokens are replaced before anything is stored or shown. You can read the whole
report before deciding.

Nothing is sent unless you choose Send, and sending is off by default. You can
save the report to a file instead.

## Not yet shipped

These are not in place, so none of them is promised here:

- **A Bug reports page in bos-settings.** Until it exists, the settings screen does not show this notice or the sending switch.
- **This statement on the BOS site.** The site's current wording is not this text, and it is still to be corrected.
- **A single consent source read by the reporter and bos-settings.** Today the text is a constant in bread's Rust code, and this page holds a copy.
- **A request path for deletion without the local token**, and removal of breadboard issue copies when a report is deleted.
- **Enforced type and size checks on event fields** at ingest and on the server. The current event code does not enforce the table's semantic types and ranges.
- **A windowed reporter (`breadbug-gui`)** is in the bread code but not wired into BOS, so the shortcut and the notification open the terminal reporter.
- **A review of this page against the shipped schema before release.**
