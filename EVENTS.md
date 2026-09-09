# breadhelp: bread event integration

breadhelp is a standalone GTK help center: it works exactly the same with
or without `breadd` running. When breadd *is* present, it publishes one
event after the main window is actually shown. See the parent `bread`
repo's `Documentation.md`, specifically its "Namespaces" and
"Integrating a bread\* app" sections, for the general convention this
follows.

App id: **`help`**. Transport: `bread-utils`'s `bread_client` module
(feature `bread-client`): breadhelp links it directly. Each `emit` is
its own short-lived connection. Command verbs are only received while
`breadhelp listen` is running; that process holds the
`bread.command.help.**` subscription open.

## Events published (`bread.help.*`)

| Event | Data | When |
|-------|------|------|
| `bread.help.opened` | `{ "autostart": bool }` | The main help window is presented (`ApplicationWindow::present`). `autostart` is `true` when that invocation was launched with `--autostart`. |
| `bread.help.open.done` | `{}` | `bread.command.help.open` was received and `breadhelp` was spawned. This is the command confirmation, not proof the window mapped: the spawned process is the same no-args invocation as SUPER+/. |
| `bread.help.open.failed` | `{ "error": "<message>" }` | `bread.command.help.open` was received but this binary could not be started. |

Not emitted when:

- every-login `--autostart` builds a hidden window because onboarding is
  already done (silent autostart)
- first-run `--autostart` presents the main window on the day-zero
  wizard; the tour overlay starts after that wizard finishes (or is
  skipped). `bread.help.opened` **is** emitted for that first-run present.
- `--onboard` / `--tour-event` (tour only)
- `--screenshot` (capture, not a user-visible open)

## Commands honored (`bread.command.help.*`)

These are only received while `breadhelp listen` is running. Publishing a
command with no subscriber is a silent no-op; that is the documented
bread convention, not a breadhelp bug.

| Verb | Data | Effect |
|------|------|--------|
| `open` | none | Same as running `breadhelp` with no flags: present the main help window (GApplication forwards to an already-running primary instance). Emits `bread.help.open.done` / `.failed`. |

```lua
bread.spawn(function()
    bread.emit("bread.command.help.open")
    bread.wait("bread.help.open.done", { timeout = 5000 })
end)
```

### Not implemented: extra verbs

There is no `onboard` / `tour` / `suggest` command verb. Those already
exist as local CLI flags (`--onboard`, `--tour-event`, `--suggest`).
If/when a bus verb maps to real extra behavior, add it then; do not
stub one as a no-op ahead of it.

## Fail-safe behavior

- If breadd isn't installed or isn't running, `emit` is a silent no-op
  (`BreadClient::emit` never blocks or errors the caller) and the
  command subscription simply never receives anything: the help
  center, tour, and screenshots are entirely unaffected.
- If breadd restarts, the command subscription reconnects automatically
  (`BreadClient::subscribe`'s background thread has its own backoff
  loop); no restart of `breadhelp listen` is needed.
- If `breadhelp listen` is not running, commands are a graceful no-op at
  the bus (no subscriber). The CLI still works.
