# Bug reports and your privacy

What the reporter collects, what it removes, and what it never touches. Checked against the shipped reporter at bread `e4ddf6c` and the server at breadbug-server `8669888` by running the real commands in an isolated directory (see Provenance). Anything the shipped code cannot do yet is under Planned, not promised.

## What a report contains

Schema version 1. These are the exact top-level sections; `breadbug --app ID --dry-run` prints them:

- App: which app, its version and build id.
- System facts: OS id and version, kernel, CPU arch, graphics drivers with PCI ids, compositor version, bakery track, breadd and API versions.
- Installed bread apps, with versions.
- Runtime: whether the daemon is running, its uptime, adapter statuses, module counts.
- Markers: up to 5 recent crash or freeze records with stack frames (function names and offsets only, never memory contents).
- Logs: recent app log lines, plus the daemon's warnings and errors.
- Events: bread event names with allowlisted fields only (battery percentage, online state, USB vendor and product ids). Anything not allowlisted is dropped and only the name is kept.
- Journal: recent `journalctl --user` lines for the app's own processes.
- Live stack: frames of a still-running frozen process, only captured with your explicit agreement.
- Your notes, scrubbed, at most 8 KiB.
- Redactions: counts of replaced values per category, so the preview shows what was removed.

The payload is capped at 1 MiB. The builder trims journal lines first, then the oldest events and logs, to fit.

## What is replaced before anything is stored or shown

- Your name, home folder and computer name.
- IP and MAC addresses, email addresses, device serial numbers, Wi-Fi network names, window titles.
- Passwords, access tokens, keys, and the credentials and query strings in web addresses.
- Long random-looking strings that look like secrets.

This happens twice: when the daemon first stores a log line, and again over the whole report with a fresh scrubber before you preview it. You read the result before deciding. Text matching is best effort: it cannot know that an arbitrary word is a name, so read the preview and treat your notes as visible to the triage team.

Limits you should know:

- Excluding an app stops the daemon storing anything for it, but the journal section is read straight from `journalctl` at report time, not from the store, so an excluded app can still appear there. Check the preview.
- Your notes are scrubbed like everything else, but they are free text, so do not paste secrets into them.

## What is never collected

- Screenshots. Process memory, registers, or environment. Your files. Your clipboard contents.

## Where it goes and how long it stays

- Nothing leaves your computer from the shipped reporter: the send step is not implemented yet, and saved files stay where you put them, readable only by you.
- Planned server behavior, for when sending lands: reports go to the bread project's own server over HTTPS. The server never stores IP addresses and re-scrubs every report without trusting the client. Raw reports are deleted after 90 days; only cluster metadata is kept (grouped issue clusters with locally computed text embeddings, used to spot duplicates, never sent to a third party). Deletion uses a token returned once and stored hashed; backups rotate, so an explicit deletion reaches backup copies on the next successful rotation, not instantly.
- There is no install id or rotation in the shipped reporter. A random install id with a fixed rotation lifetime exists only on the unmerged upload branch, so until that lands no such promise applies.

## Your controls today

- Save a report to a file instead of sending it.
- `breadbug config upload on` or `off` records whether you allow sending. The default is off, and with it off nothing touches the network. Allowing it changes nothing yet, because sending is not implemented (see the reporting guide's Planned section).
- Exclude an app in `breadbug.toml` so the daemon stores nothing for it, keeping the journal caveat above in mind.

## Provenance: one text, three places, not yet

HELP-10 requires this statement to be the same text the reporter shows and the settings page shows, and to be published on the BOS site. That sharing is not implemented: the consent text lives only as a proposal on bread `feature/breadbug-consent`, there is no Bug reports page in bos-settings, and the BOS site has no privacy page. Inside this repo the wording is kept once in `content/shared/breadbug-consent.md` and quoted below, so the guides cannot drift apart while the cross-repo work is pending.

Reporting a problem

A report contains your app's recent log lines, crash frames, event names and
basic system facts such as OS, kernel, GPU driver and bread versions. Personal
values such as your name, home folder, hostname, IP addresses, emails and
tokens are replaced before anything is stored or shown. You can read the whole
report before deciding.

Nothing is sent unless you choose Send, and sending is off by default. You can
save the report to a file instead.
