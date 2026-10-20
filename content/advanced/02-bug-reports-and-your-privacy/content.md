# Bug reports and your privacy

What breadbug collects, what it removes, what it never touches, and what happens when you send a report. This page describes the code on bread `main` (the merged breadbug stack). Anything that depends on work not yet merged is under Not yet shipped, not promised.

## What a report contains

Schema version 1. These are the top-level sections of a report. `breadbug --app ID --dry-run` prints the exact report:

- App: which app, its version and build id.
- System facts: OS id and version, kernel, CPU architecture, graphics drivers with PCI ids, compositor version, bakery track, breadd and API versions.
- Installed bread apps, with their versions.
- Runtime: whether the daemon is running, its uptime, adapter statuses and module counts.
- Markers: up to the 5 most recent crash or freeze records, with stack frames. Frames are function names and offsets only, never memory contents.
- Logs: recent app log lines, plus the daemon's warnings and errors.
- Events: bread event names with allowlisted fields only (battery percentage, online state, USB vendor and product ids). Anything not allowlisted is dropped, and only the name is kept.
- Journal: recent `journalctl --user` lines for the app's own processes.
- Live stack: frames of a still-running frozen process. Only captured with your explicit agreement.
- Your notes, scrubbed, at most 8 KiB.
- Redactions: counts of replaced values per category, so the preview shows what was removed.

The report is capped at 1 MiB. The builder trims journal lines first, then the oldest events and logs, to fit.

## What is replaced before anything is stored or shown

- Your name, home folder and computer name.
- IP and MAC addresses, email addresses, device serial numbers, Wi-Fi network names and window titles.
- Passwords, access tokens, keys, and the credentials and query strings in web addresses.
- Long random-looking strings that look like secrets.

This happens twice: when the daemon first stores a log line, and again over the whole report with a fresh scrubber before you preview it. You read the result before deciding. Text matching is best effort. It cannot know that an arbitrary word is a name, so read the preview, and treat your notes as visible to the people who read the report.

Limits you should know:

- Excluding an app in `breadbug.toml` stops the daemon storing anything for it. The journal section is read straight from `journalctl` when the report is built, not from the store, so an excluded app can still appear there. Check the preview.
- Your notes are scrubbed like everything else, but they are free text, so do not paste secrets into them.

## What is never collected

- Screenshots.
- Process memory, registers or environment variables.
- Core dumps.
- Your files.
- Your clipboard contents.

Live stack captures contain module names, module-relative offsets and build ids only. They never include memory, registers, environment variables or a core dump.

## Sending is off by default

Sending needs two things: `upload_enabled = true` (set with `breadbug config upload on`) and a separate, explicit Send for that one report. The setting is off by default. While it is off, breadbug does not touch the network, including the queue, history refresh and delete.

Saving a report to a file never needs the network and is always allowed.

## Where a sent report goes

- Reports go over HTTPS to the endpoint in `breadbug.toml`. The default is the bread project's intake service. Plain HTTP is only accepted for a local loopback address. Redirects are never followed.
- The exact previewed bytes are sent, unchanged. The client checks their hash right before sending, and the server checks it too.
- The server re-scrubs every report. It does not trust the client's scrubbing.

What the intake service keeps, how long it keeps it, and how deletion works on its side are its own policy, not something the reporter enforces. The server's documented policy is in the breadbug-server repository (`docs/threat-model.md` and `docs/feedback-api.md`). It describes raw reports being deleted after 90 days, with only cluster metadata kept, and an explicit deletion removing a report from the next backup rotation. The server documents that it does not store IP addresses. It also documents that an unmanaged backup copy outside its control is not erased by it.

## Install id

- The install id is a random 128-bit number, created on first use and stored only on this machine in `~/.local/state/bread/breadbug/install.json`.
- It is replaced every 30 days.
- There is no account and no stable device identifier.
- The server receives it in a header on each send. It stores only a keyed hash of it.

## Queue, history and receipts

- If the server cannot be reached, the exact report is saved to an offline queue of up to 20 reports and retried with increasing waits, up to 6 hours. See the queue with `breadbug queue`, retry with `breadbug queue flush`, and drop reports with `breadbug queue cancel` or `breadbug queue clear`.
- A report the server rejects is dropped. Nothing more is tried for it.
- A successful send returns a receipt and a deletion token. Both are kept in your local history (`history.jsonl`, readable only by you).

## Your controls

- Save a report to a file instead of sending it.
- Turn sending on or off with `breadbug config upload on` or `off`.
- Exclude an app in `breadbug.toml`, so the daemon stores nothing for it. Keep the journal caveat above in mind.
- See your sent and saved reports with `breadbug history`.
- Ask the server how your sent reports are doing with `breadbug status`. It only checks receipts you hold.
- Delete a report you sent with `breadbug delete <receipt>`. This asks the server to delete it, then removes the entry from your local history. It needs sending to be on, and it needs the deletion token kept on this machine. A report recognised as an earlier identical send has no token here, so it cannot be deleted from this machine.

## Consent notice

The notice below is the text breadbug shows before your first session. It is the same text as the `CONSENT_TEXT` constant in bread `bread-bug/src/consent.rs`.

Reporting a problem

A report contains your app's recent log lines, crash frames, event names and
basic system facts such as OS, kernel, GPU driver and bread versions. Personal
values such as your name, home folder, hostname, IP addresses, emails and
tokens are replaced before anything is stored or shown. You can read the whole
report before deciding.

Nothing is sent unless you choose Send, and sending is off by default. You can
save the report to a file instead.

## Not yet shipped

These are not in place yet, so none of them is promised here:

- A Bug reports page in bos-settings. Until it exists, the settings screen does not show this notice or the sending switch.
- This statement published on the BOS site. The site's current wording on this topic is not the same text as the notice above, and it is still to be corrected.
- A single consent source read by the reporter and bos-settings. Until it exists, the notice is copied in each place.
- A fresh review of this page against the shipped report schema, before release.
