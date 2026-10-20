# Reporting a problem

If a bread app crashed, froze, or isn't behaving, you can build a report, read exactly what it contains, and then save it to a file or send it. Nothing is sent unless you choose Send, and sending is off by default.

- [Terminal]breadbug|Report a problem[/Terminal]

The button opens a terminal running `breadbug`. The reporter is a terminal program: it needs a keyboard and screen of its own, so this help window launches it in the BOS terminal rather than as a bare command.

## Report a crash or a freeze

- Press **Super + Shift + B**, or wait for the notification after a bread app crashes or freezes and choose **Review report**.
- Pick the app that has the problem. Apps that recently crashed or froze are listed first.
- Say what happened: a crash, a freeze, or something behaving wrongly. You can add a sentence in your own words. Do not paste passwords or private text. It is scrubbed, but you are the best judge of what is private.

## What the preview shows

Before anything can be sent, breadbug shows the report:

- Counts for each part of the report: crash and freeze records, app log lines, daemon warnings, event names, journal lines and installed apps.
- What the scrubber replaced, by category, such as names, addresses and tokens.
- A SHA-256 hash of the exact bytes. Choose **v** (view everything) to see those bytes.

## Save a report to a file

Choose **f** (save to a file) and give a path, or run `breadbug --app clip --save report.json`. This writes the exact previewed bytes, readable only by you, and prints the hash. Nothing is sent.

## Send a report

Sending needs two things: sending switched on, and an explicit Send for that report.

- Turn sending on with `breadbug config upload on`. It is off by default.
- Choose **s** (send) in the preview, or run `breadbug --app clip --send`.

A sent report returns a receipt, which breadbug prints. Keep it if you want to delete the report later.

If the server cannot be reached, the report is saved to an offline queue and retried later. See it with `breadbug queue`.

## Delete a report you sent

- `breadbug history` lists the reports you sent or saved, with their receipts.
- `breadbug delete <receipt>` asks the server to delete the report and then removes it from your history. It needs sending to be on, and it works only on the machine that sent the report.

The guide "Bug reports and your privacy" lists exactly what is collected and how long it is kept.

## Check on your reports

- `breadbug status` asks the server how your sent reports are doing, and tells you when one is fixed in a version newer than yours. It only checks receipts you hold.

## Without the menu

These work in any terminal, with nothing interactive needed:

- `breadbug list` shows the apps you can report on.
- `breadbug --app clip --dry-run` prints the exact report that would be sent, then exits.
- `breadbug --app clip --save report.json` writes it to a file. Nothing is sent.

Run bare `breadbug` with no terminal attached and it stops with an error telling you to pass `--app`. That is expected: the app picker needs a terminal.

## Not yet shipped

- A windowed reporter (`breadbug-gui`) is in the bread code but is not wired into BOS yet, so the shortcut and the notification open the terminal reporter.
- A Bug reports page in bos-settings is not there yet, so sending is switched on from the terminal.
