# Reporting a problem

If a bread app crashed, froze, or just isn't behaving, you can build a report, read exactly what it contains, and save it to a file. Nothing is sent unless you choose to send it, and sending is not available in this build yet (see Planned below).

- [Terminal]breadbug|Report a problem[/Terminal]

The button opens a terminal running `breadbug`. The reporter is a terminal program: it needs a keyboard and screen of its own, and this help window cannot give it one, so it is launched in the BOS terminal instead of bare.

## What you'll do

- Pick the app that has the problem. Apps that recently crashed or froze are listed first.
- Say what happened: a crash, a freeze, or something behaving wrongly. You can add a sentence in your own words. Do not paste passwords or private text; it is scrubbed, but you are the best judge of what is private.
- Read the report summary. It shows counts per section, what the scrubber replaced, and a SHA-256 hash. Ask to view everything to see the exact bytes that would be sent.
- Choose **save to a file**, **send**, or **cancel**. Save writes the exact previewed bytes (readable only by you) and prints the hash. Send is offered but answers that sending is not available in this build yet, so nothing leaves your computer either way.

## Without the menu: list, preview, save

These work in any terminal today, with nothing interactive needed:

- `breadbug list` shows the apps you can report on.
- `breadbug --app clip --dry-run` prints the exact report that would be sent, then exits.
- `breadbug --app clip --save report.json` writes it to a file and prints its hash. Nothing is sent.
- `breadbug --app clip --send` exists, but with sending off it tells you how to turn it on, and with sending on it tells you sending is not available in this build yet.

Run bare `breadbug` with no terminal attached and it stops with an error telling you to pass `--app`. That is expected: the app picker needs a terminal.

## Sending stays off until sending exists

`breadbug config upload on` records that you allow sending; `off` forbids it. The default is off, and with it off nothing in breadbug touches the network. But allowing it sends nothing yet: the send step on shipped code only saves to a file, because the upload client lives on an unmerged branch (see Planned).

## Planned: needs unmerged work first

The following are designed and partly built on bread branches, but none of them is in the shipped reporter yet. Do not expect them to work:

- Sending a report to the server, receipts, `breadbug status`, `breadbug delete`, the offline queue (`breadbug queue`), and fix notifications. Branches: `feature/breadbug-upload`, `feature/breadbug-status`.
- A windowed reporter, and a crash notification with a review action. Branch: `feature/breadbug-gui`.
- A `super + shift + b` shortcut. BOS has no breadbug keybind.
- The shared consent notice before the first report, shown in the reporter and in settings. Branch: `feature/breadbug-consent`. The proposed wording is kept with the privacy guide until that lands.
- A Bug reports page in bos-settings, and this statement published on the BOS site. Neither exists yet; both are tracked as prerequisites for HELP-10, not claimed here.
