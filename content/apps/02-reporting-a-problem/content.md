# Reporting a problem

If a bread app crashed, froze, or isn't behaving, you can build a report, read exactly what it contains, then save it to a file or send it. Nothing is sent unless you choose Send, and sending is off by default.

- [Terminal]breadbug|Report a problem[/Terminal]

The button opens a terminal running `breadbug`. The reporter is a terminal program, so this help window launches it in the BOS terminal rather than as a bare command. On a machine with breadbug from the bakery dev track, you can also press **Super + Shift + B**. The crash notification offers a **Review report** action that starts the same reporter.

## Which builds have it

Breadbug is in bread `v0.10.0` and on bread `main`. The bakery dev track builds from bread `main`, so machines on that track already have breadbug and the daily status timer. The current BOS ISO pin is still bread `0.9.0`, so an image built from that pin does not include breadbug. The shortcut and the crash notification are on bos `main`, but no BOS tag up to `v0.6.2` has them, so they are not available on a stable BOS image yet.

## Report a crash or a freeze

- Open the reporter from this page. On a machine with breadbug from the bakery dev track, you can also press **Super + Shift + B**.
- Pick the app that has the problem. Apps that recently crashed or froze are listed first.
- Choose what happened: crash, freeze, or behavior.
- Add a note in your own words if you want to. Do not paste passwords or private text. It is scrubbed, but you are the best judge of what is private.

The first time you run the interactive reporter, breadbug shows a short notice and waits for Enter. Press `q` to stop without recording that you read it. It shows again next time. The notice is not shown when you pass `--app`, `--save`, `--send` or `--dry-run`.

## What the preview shows

Before anything can be sent, breadbug prints a summary of the report:

- The app and the report type.
- A SHA-256 hash of the exact report bytes.
- The size of the report.
- Counts for each part of the report: crash and freeze records, app log lines, daemon warnings, event names, journal lines, live stack frames and installed apps.
- Your note, if you wrote one.
- What the scrubber replaced, by category, for example `2 IP, 1 USER`. If it replaced nothing, it says so. This count covers only the values replaced when the report was built. Values the daemon already removed when it stored them are not counted, so a report can show no replacements even though IP addresses or names were removed earlier. See the privacy guide.

Then it asks what you want to do:

- **v** shows the whole report, which is the exact text that would be sent.
- **f** saves it to a file.
- **s** sends it, if sending is on.
- **c** cancels. Nothing is sent or saved.

## Save a report to a file

Choose **f**, then give a file name. Pressing Enter accepts the default, `breadbug-APP-UNIXTIME.json`, where the last part is the report's creation time in seconds since 1970. The file is written in the folder you ran it from, unless you give a path.

Breadbug writes the exact previewed bytes and prints the hash. Nothing is sent. A save is recorded in your local history.

File mode: breadbug writes the report to a fresh file with mode 0600 and renames it into place, so a new file and a replaced file are both readable only by you. In the menu, if the file already exists, breadbug asks before overwriting it. `breadbug --app clip --save report.json` replaces an existing file without asking, and still leaves it mode 0600.

## Send a report

Sending needs two things: sending switched on, and an explicit Send for that report.

- Turn sending on with `breadbug config upload on`, or in bos-settings under Bug reports. It is off by default.
- In the preview, choose **s**. Or run `breadbug --app clip --send`, which prints the summary and sends it.

A successful send prints a receipt. It also prints the `breadbug delete` command when the server gave a deletion token. Keep the receipt if you want to delete the report later.

If the server cannot be reached, the exact report is kept in an offline queue. See it with `breadbug queue`. Nothing retries it on its own. Send the queue with `breadbug queue flush`, which tries every queued report at once. A report the server rejects as invalid is dropped from the queue; a stopped marker is not sent, because the client removes it when it builds the report.

## Delete a report you sent

- `breadbug history` lists the reports you sent or saved, with their receipts.
- `breadbug delete <receipt>` asks the server to delete the report, then removes it from your history.

Two conditions apply:

- Sending must be on, because `delete` contacts the server.
- It works only on the machine that sent the report, because the deletion token is kept in your local history and nowhere else. If you sent the same report twice, the second send returns the original receipt and no new token. `breadbug delete` still works with that receipt, because breadbug finds the first history entry, which holds the token. It stops working only when that entry has been dropped: breadbug keeps the newest 200 history entries.

The guide "Bug reports and your privacy" says exactly what is collected, what the server keeps, and what a delete does not remove.

## Check on your reports

- `breadbug status` asks the server how your sent reports are doing. It tells you when one is fixed in a version newer than yours. It only checks receipts you hold, and it needs sending to be on.
- The bakery timer runs `breadbug status` once a day. With sending on and at least one receipt, that call contacts the server for those receipts. It never sends a report.

## Without the menu

These work in any terminal, with nothing interactive needed:

- `breadbug list` shows the apps you can report on.
- `breadbug --app clip --dry-run` prints the exact report that would be sent, then exits.
- `breadbug --app clip --save report.json` writes it to a file. Nothing is sent.

Run bare `breadbug` with no terminal attached and it stops with an error telling you to pass `--app`. That is expected: the app picker needs a terminal.

## Not yet shipped

- **BOS site publication.** The BOS site does not carry this text or the privacy statement yet.
- **Deletion without the local token.** There is no request path for someone who has lost the machine that sent the report.
- **Server-side enforcement of the event allowlist and field types.** The sending client drops fields outside the list; the server does not enforce the same list.
