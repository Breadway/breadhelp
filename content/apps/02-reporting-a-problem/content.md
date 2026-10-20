# Reporting a problem

If a bread app crashed, froze, or isn't behaving, you can build a report, read exactly what it contains, then save it to a file or send it. Nothing is sent unless you choose Send, and sending is off by default.

- [Terminal]breadbug|Report a problem[/Terminal]

The button opens a terminal running `breadbug`. The reporter is a terminal program, so this help window launches it in the BOS terminal rather than as a bare command.

## Report a crash or a freeze

- Press **Super + Shift + B** to open the reporter.
- Pick the app that has the problem. Apps that recently crashed or froze are listed first.
- Choose what happened: crash, freeze, or behavior.
- Add a note in your own words if you want to. Do not paste passwords or private text. It is scrubbed, but you are the best judge of what is private.

The first time you run it, breadbug shows a short notice and waits for Enter. Press `q` to stop without recording that you read it. It shows again next time.

## What the preview shows

Before anything can be sent, breadbug prints a summary of the report:

- The app and the report type.
- A SHA-256 hash of the exact report bytes.
- The size of the report.
- Counts for each part of the report: crash and freeze records, app log lines, daemon warnings, event names, journal lines, live stack frames and installed apps.
- Your note, if you wrote one.
- What the scrubber replaced, by category, for example `2 IP, 1 USER`. If it replaced nothing, it says so.

Then it asks what you want to do:

- **v** shows the whole report, which is the exact text that would be sent.
- **f** saves it to a file.
- **s** sends it, if sending is on.
- **c** cancels. Nothing is sent or saved.

## Save a report to a file

Choose **f**, then give a file name. Pressing Enter accepts the default, `breadbug-APP-UNIXTIME.json`, where the last part is the report's creation time in seconds since 1970. The file is written in the folder you ran it from, unless you give a path.

Breadbug writes the exact previewed bytes and makes the file readable only by you (mode 0600). It prints the hash. Nothing is sent. A save is recorded in your local history.

You can also save without the menu: `breadbug --app clip --save report.json`.

## Send a report

Sending needs two things: sending switched on, and an explicit Send for that report.

- Turn sending on with `breadbug config upload on`. It is off by default.
- In the preview, choose **s**. Or run `breadbug --app clip --send`, which prints the summary and sends it.

A successful send prints a receipt. It also prints the `breadbug delete` command when the server gave a deletion token. Keep the receipt if you want to delete the report later.

If the server cannot be reached, the exact report is kept in an offline queue. See it with `breadbug queue`. Send the queue with `breadbug queue flush`. A report the server rejects is dropped.

## Delete a report you sent

- `breadbug history` lists the reports you sent or saved, with their receipts.
- `breadbug delete <receipt>` asks the server to delete the report, then removes it from your history.

Two conditions apply:

- Sending must be on, because `delete` contacts the server.
- It works only on the machine that sent the report. The deletion token is kept in your local history and nowhere else. A report that the server recognised as an identical earlier send has no token here, so it cannot be deleted from this machine.

The guide "Bug reports and your privacy" says exactly what is collected, what the server keeps, and what a delete does not remove.

## Check on your reports

- `breadbug status` asks the server how your sent reports are doing. It tells you when one is fixed in a version newer than yours. It only checks receipts you hold, and it needs sending to be on.

## Without the menu

These work in any terminal, with nothing interactive needed:

- `breadbug list` shows the apps you can report on.
- `breadbug --app clip --dry-run` prints the exact report that would be sent, then exits.
- `breadbug --app clip --save report.json` writes it to a file. Nothing is sent.

Run bare `breadbug` with no terminal attached and it stops with an error telling you to pass `--app`. That is expected: the app picker needs a terminal.

## Not yet shipped

- A windowed reporter (`breadbug-gui`) is in the bread code, but it is not wired into BOS yet. The shortcut and the notification open the terminal reporter.
- A Bug reports page in bos-settings is not there yet, so sending is switched on from the terminal.
