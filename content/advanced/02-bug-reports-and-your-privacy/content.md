# Bug reports and your privacy

This page says what breadbug collects, what it removes, what it never touches, and what happens when you send a report. It describes the breadbug client on bread `main` (commit 86ac332) and the server on breadbug-server `main` (commit b5d877e), checked on 2026-11-07 by reading that code and the shipped `docs/breadbug.md`. Anything not in place yet is listed under Not yet shipped and is not promised.

## Release status

Breadbug is in bread `v0.10.0` and on bread `main`. The bakery dev track builds from bread `main`, so machines on that track already have breadbug and the daily status timer. The current BOS ISO pin is still bread `0.9.0`, so an image built from that pin does not include breadbug. The `Super + Shift + B` shortcut and the crash notification are on bos `main`, but no BOS tag up to `v0.6.2` has them.

To check later, run `git tag --contains 34dda59` in bread (34dda59 is the merge that added breadbug) and `git grep -i breadbug <tag> -- iso/airootfs/etc/skel/.config/hypr/binds.json` in bos. If either answer changes, update this page.

## What a report contains

Schema version 1. These are the top-level sections. `breadbug --app ID --dry-run` prints the exact report, so you can read it before deciding.

- **App**: the app id, its installed version and its build id, when known.
- **System facts**: OS id and version, OS build, kernel, CPU architecture, GPU driver with PCI vendor and device ids, compositor version, bakery track, and breadd and API versions.
- **Installed bread apps**: app ids and versions for the bread apps bakery reports as installed.
- **Runtime**: whether the daemon is running, its uptime, adapter status strings, and module counts.
- **Markers**: up to the 5 most recent records. Each has a time, an optional message and location, and up to 128 stack frames. Markers are usually panic, crash, freeze and recovery records. A stopped record (the app's process ended) is kept locally to close a freeze, and the client removes stopped markers before building the report. An older client can still send one; the server accepts and stores it so that report is not lost, but it never builds or joins a cluster.
- **Logs**: up to 300 recent log lines from the app.
- **Daemon logs**: the daemon's recent warnings and errors.
- **Events**: bread event names. Only these fields survive: battery percentage and AC state, network online state, and device or Bluetooth subsystem, vendor and product ids. Other fields are dropped, and only the event name is kept.
- **Journal**: recent `journalctl --user` lines from the app's own processes, up to 150 lines. Each line is cut at 2048 bytes.
- **Live stack**: stack frames of a frozen app that is still running. Only present if you agreed to that capture for this report.
- **Notes**: the note you typed, scrubbed, at most 8 KiB.
- **Redactions**: how many values the scrubber replaced in each category when the report was built.

A report is capped at 1 MiB. If it is too large, the builder drops journal lines first, then the oldest events and logs.

## What is replaced before anything is stored or shown

The scrubber replaces values with markers such as `<HOME>`, `<IP:1>` or `<SECRET>`. A number after the name separates different values of the same kind.

- Your user name and computer name (`<USER>`, `<HOST>`) and your home folder (`<HOME>`).
- IP addresses (`<IP>`), MAC addresses (`<MAC>`), email addresses (`<EMAIL>`), device serial numbers (`<SERIAL>`) and UUID-shaped values (`<UUID>`).
- Wi-Fi network names (`<SSID>`) and window titles (`<TITLE>`) where they appear as `ssid=`, `title=` or `window_title=` style values in text.
- Passwords, secrets and private keys (`<SECRET>`). Recognised token formats are also replaced with `<SECRET>`: GitHub, GitLab, Slack, AWS, Google and `sk-` keys, JWTs, and `Bearer` or `Basic` values. Values after `password=`, `token=`, `api_key=` and similar keys are `<SECRET>` too.
- Credentials in web addresses (`<CRED>`) and query strings (`<QUERY>`).
- Long random-looking strings are `<TOKEN>`, and long hex strings are `<HEX>`.
- Build ids are exempt from the hex and random-string rules, because symbol lookup needs them.

Serial numbers are replaced only after a `serial`, `serial number` or `serial no` key (for example `serial=...`). A bare serial, such as one in a disk model string, is not matched.

Scrubbing runs in two places:

- When the daemon stores a log line, a marker message, or an event name and its allowed fields.
- When the report is built, over the whole report with a new scrubber. That pass also covers notes and the journal, which are not scrubbed when stored.

The preview shows how many values the report-time pass replaced in each category. Values already replaced when they were stored are not counted again. So the preview can say it replaced nothing even when IP addresses and names were removed earlier. The counts are not a full record of what was removed.

The report-time pass uses your user name, host name and home folder. The server's re-scrub is given none of them and uses only the pattern rules, so it cannot catch a literal user or computer name that the client missed.

Limits you should know:

- Scrubbing is best effort, and it works on text patterns. It cannot know that an ordinary word is a person's name. Read the preview.
- Your notes are scrubbed like everything else, but they are free text. Do not paste passwords or private text into them.
- Event fields are allowlisted on the client, so event payloads outside the list are dropped. The server does not enforce that list (see Not yet shipped). Free text in logs, markers and the journal is only pattern-scrubbed.
- The journal is read directly from `journalctl` when the report is built, not from breadbug's store. Excluding an app (see below) does not remove its journal lines from a report. Check the preview.

## What is never collected

These are the things the code does not read or send:

- **Screenshots.** No code in breadbug, bread-bug or the scrubber captures the screen.
- **Process memory contents, registers and environment variables.** A live stack is taken with `eu-stack`, which attaches to the frozen process and reads its stack to unwind it. The parser keeps only module names, module-relative offsets, build ids and symbol names. No memory contents, registers or environment are kept or sent.
- **Core dumps.** No code in bread's breadbug, bread-bug, breadd or bread-scrub crates reads or writes core dumps.

These are not collected by the report schema above. They are not guaranteed to be absent from every log line an app writes, so do not rely on this list alone and read the preview:

- **Memory contents** of apps, other than what an app itself writes to its logs.
- **Window titles and personal files.** No report section is built from window titles or from the contents of files in your home folder. A window title or file path can still appear in a log line or the journal if an app printed it there, and the scrubber only removes it when it matches a pattern.

Breadbug has no default exclusion for apps that handle clipboard contents or search queries. Exclusion is opt-in in `breadbug.toml` (`excluded_apps`, empty by default). The code has no list of sensitive apps yet.

## Sending is off by default

Sending needs two things: `upload_enabled = true` (set with `breadbug config upload on`), and a separate explicit Send for that one report. The setting is off by default. While it is off, breadbug does not contact the network. That includes the queue, `history --refresh`, `status` and `delete`.

Saving a report to a file never needs the network, and it is always allowed.

The bakery-installed timer runs `breadbug status` once a day (see Queue and receipts). With sending on and at least one receipt, that call contacts the server for each receipt you hold. It does not send any queued report.

## Where a sent report goes

- Reports go over HTTPS to the endpoint in `breadbug.toml`. The default is `https://bugs.breadway.dev`.
- Plain HTTP is accepted only for a literal loopback address: `127.0.0.1`, `::1` or the host name `localhost`. A host name that merely begins with `localhost` or `127.0.0.1` is refused.
- Redirects are never followed.
- The exact previewed bytes are sent unchanged. The client checks their hash right before sending. The server checks the `X-Breadbug-Sha256` hash too.
- The server scrubs every report again with the pattern rules. A report with anything new to redact is stored scrubbed and quarantined, which keeps it out of clustering until someone reviews it.
- Reports are read by maintainers in a non-public triage inbox. Quarantined reports are reviewable there too.
- Reports sent with the same install id share one keyed install hash, so the server can link them to each other. The install id is replaced after 30 days, which changes the hash. The server never stores the install id itself (see Install id).

## What the server keeps, and for how long

This section describes the breadbug-server code, not anything breadbug on your machine enforces.

- **Raw reports** are deleted once they are older than the retention period. The default is 90 days (`BREADBUG_RETENTION_DAYS`). A purge runs every hour.
- **Cluster metadata** is kept while the cluster exists. A cluster is the server's grouping of similar reports. It holds:
  - the app, the kind (crash, freeze or behavior), and counts, versions and OS builds, with first and last seen times;
  - a title taken from report text. A crash or freeze that has a stack or a source location is titled from the crash: the normalised message, or the kind with a frame name or location. Every other report, including every behavior report, a crash with no frames and no location, and a freeze without a live stack, is titled with the first line of your note (up to 100 characters), or otherwise a marker message or a warning or error log line. So a line from your note can end up in the title;
  - an embedding vector (a centroid of the reports in the cluster), used to group similar reports;
  - the install hashes of the reports in the cluster (membership only, not the install id);
  - the breadboard issue link and status, if the cluster was promoted.
- **Grouping is done on the server, not by an outside service.** Similar reports are grouped by a built-in local embedder (a hashing embedder that runs in the server process). The server can be pointed at an embedding service with `BREADBUG_EMBED_URL`, but only a plain `http://` URL whose host resolves only to loopback, private IPv4 or IPv6 unique-local addresses is accepted. The address is checked when the client is built and again on every call. Any other value is refused and the built-in embedder is used instead. The server makes no calls to third-party model APIs. The only outbound network clients in the server are the breadboard connection and the embedding client.
- **Install ids** are stored only as a keyed hash. The server does not store the install id itself.
- **IP addresses** are not stored. The server's network bucket is a keyed hash held only in memory for rate limiting, and it is never persisted. The reports table has no address column.
- **Logs**: the threat model says report content is never written to logs, only counts and errors.

Not guaranteed by the code in this repository:

- Whether the reverse proxy in front of the server keeps access logs. Infra `main` configures the `bugs.breadway.dev` block with `log { output discard }`, so the repository's intent is no access log. This page did not check the deployed proxy.
- What happens to a report in a backup. Backups are kept by the operator. A deleted report stays in a backup until that backup is rotated (`README.md`, backup and deletion contract). Copies made outside the tool's control are not erased by it.

## Deleting a report you sent

When a send succeeds, the server returns a receipt and a deletion token. breadbug keeps both in `history.jsonl`, readable only by you.

- `breadbug history` lists the reports you sent or saved, with their receipts.
- `breadbug delete <receipt>` asks the server to delete that report, using the deletion token kept on this machine. On success, breadbug removes the entry from your history. It needs sending to be on.
- Sending the same report again does not give a new token. The server returns the original receipt with no token. breadbug finds the first history entry for that receipt, which still holds the original token, so `breadbug delete` works from the machine that sent it. The real limit is history size: breadbug keeps only the newest 200 history entries. Once the entry with the token has been dropped, a duplicate send can no longer be deleted from that machine.

What a server-side delete removes: the raw report row is deleted, and its contribution to the cluster counts, versions, OS builds and install membership is taken back out. A fuzzy cluster's title and embedding centroid are rebuilt from the reports that remain. If no report remains, an unacted cluster is deleted. A cluster someone acted on keeps its issue or merge link but gets a neutral title and loses its vector. The database is then compacted with secure delete on (`src/db.rs`, `delete_report`, `refresh_fuzzy_content` and `compact`; `src/app.rs`, `delete_report`).

What it does not remove:

- **Copies in backups**, until the backup is rotated.
- **A crash cluster's title when the cluster is kept.** A fingerprint cluster's title comes from the crash signature, not from your note, and it survives after the cluster's reports are gone if the cluster is kept because someone acted on it. An empty cluster nobody acted on is deleted, including a crash cluster.
- **Breadboard issues**. Who can read a promoted issue depends on the breadboard project and its Forgejo repository. This page does not check that. When a cluster is promoted to a breadboard issue, the issue is titled `[breadbug]` followed by the cluster title. Its body includes up to three evidence samples from the cluster's reports. Each sample holds the report's title and `want` text (the breadbug client sets neither), its last marker message and up to 8 frames, up to 8 live stack symbols, and the first 300 characters of the user's note. Deleting a raw report does not edit or delete that issue, because the server has no code for that.

To ask for a report to be removed when `breadbug delete` is not available to you, there is no request path yet. See Not yet shipped.

## Install id

- The install id is 16 random bytes from `/dev/urandom`, written as 32 hex characters. It is created on first use and stored only on this machine, in `~/.local/state/bread/breadbug/install.json`.
- It is replaced when it is older than 30 days (`INSTALL_ID_LIFETIME_SECS` in `bread-bug/src/upload.rs`).
- There is no account and no device identifier. The app does not read a hardware serial or machine id.

## Queue and receipts

- If the server cannot be reached, or returns a server error or 429, the exact report is saved to an offline queue of up to 20 reports (`MAX_QUEUED`). Queued reports are stored with mode 0600 in a 0700 directory at `~/.local/state/bread/breadbug/queue`, and they stay there until you send, cancel or clear them.
- Nothing retries a queued report by itself. `breadbug queue flush` sends every queued report immediately, ignoring the stored wait. A report is not sent automatically at any other time.
- The bakery timer runs `breadbug status` once a day. It only asks the server about receipts you hold. It does not send queued reports (`packaging/systemd/breadbug-status.service`, `breadbug/src/lib.rs`, `refresh_statuses`).
- If the server rejects a report as invalid, the report is dropped from the queue and not tried again.
- Use `breadbug queue` to see what is waiting, and `breadbug queue cancel <id>` or `breadbug queue clear` to drop reports.

## Your controls

- Save a report to a file instead of sending it.
- Turn sending on or off with `breadbug config upload on` or `off`.
- Exclude an app in `breadbug.toml` so the daemon stores nothing for it. The journal caveat above still applies.
- Read your history with `breadbug history`.
- Check on your sent reports with `breadbug status`. It asks the server only about receipts you hold, and it needs sending to be on.
- Delete a report you sent with `breadbug delete <receipt>`.

## Consent notice

The notice breadbug shows before your first interactive session is the `CONSENT_TEXT` constant in bread `bread-bug/src/consent.rs`. That constant is the single source: the terminal reporter, the windowed reporter and the Bug reports page in bos-settings all read it. This page and breadhelp's shared file (`content/shared/breadbug-consent.md`) each keep a copy. A breadhelp test checks this page against the shared file. Nothing in the test reads bread's constant. The shared file is compared with bread's constant by `scripts/check-consent-sync.py`, which needs a bread checkout and is run by hand when the text changes.

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

- **BOS site publication.** The BOS site does not carry this text yet. Its `main` has no breadbug or bug report text.
- **A request path for deletion without the local token**, and removal of breadboard issue copies when a report is deleted.
- **Server-side enforcement of the event allowlist and of event field types and ranges.** The server bounds nesting depth, key count and string size only. The allowlist is enforced by the sending client, so a report from any other client is not limited to these fields.
