# Reporting a problem

If a bread app crashed, froze, or just isn't behaving, you can report it. You decide what happens next; nothing is sent unless you choose to send it.

- [Show Keybind]super + shift + b[/Show Keybind] opens the reporter any time. When an app crashes or freezes, a notification offers to open it for you.

- [Run]breadbug|Report a problem[/Run]

## What you'll do

- Pick the app that has the problem. Apps that recently crashed or froze are at the top.
- Say what happened: a crash, a freeze, or something behaving wrongly. You can add a sentence in your own words.
- Read the report. It shows exactly what would be included and how many personal values were replaced. Nothing has been sent yet.
- Choose **send**, **save to a file**, or **cancel**.

## Sending is off until you turn it on

Out of the box breadbug only lets you save a report to a file. To allow sending, turn it on in bos-settings (Bug reports), or run `breadbug config upload on`. Even then, each report is sent only when you choose Send.

## After you send

You get a receipt number. It lets you check how your report is doing (`breadbug status`), and delete it from the server if you change your mind (`breadbug delete`). If you opt in, breadbug can notify you when a fix is available as an update.

## If sending fails

If the server can't be reached, the report waits in a queue on your computer and is retried later. You can see it with `breadbug queue`, or drop it.

## If the desktop itself is broken

Open a text console, log in, and run `breadbug`. It works from a terminal and doesn't need the desktop or the bread daemon to be running.
