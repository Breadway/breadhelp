# breadhelp: bread event integration

breadhelp is a standalone help center: it works exactly the same with
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
| `bread.help.opened` | `{ "autostart": bool }` | The help window is shown. `autostart` is `true` when that invocation was launched with `--autostart` (first-run setup). |
| `bread.help.hidden` | `{ "resident": bool }` | The help window left the screen. `resident: true`: it is hidden and the process stays loaded (the opt-in resident setting), so `bread.command.help.unload` would free it. `resident: false`: the process is quitting. |
| `bread.help.unload.done` | `{}` | `bread.command.help.unload` was received and `breadhelp --unload` was spawned. |
| `bread.help.unload.failed` | `{ "error": "<message>" }` | `bread.command.help.unload` was received but `breadhelp --unload` could not be started. |
| `bread.help.open.done` | `{}` | `bread.command.help.open` was received and `breadhelp` was spawned. This is the command confirmation, not proof the window mapped: the spawned process is the same no-args invocation as SUPER+/. |
| `bread.help.open.failed` | `{ "error": "<message>" }` | `bread.command.help.open` was received but this binary could not be started. |

Not emitted when:

- every-login `--autostart` keeps the window hidden because setup and
  the tour are already done (silent autostart)
- first-run `--autostart` shows the window on first-run setup; the tour
  overlay (`breadhelp-tour`) starts after setup finishes (or is skipped).
  `bread.help.opened` **is** emitted for that first-run show.
- `--onboard` / `--tour-event` (tour only: handled before any window
  exists, and a `--tour-event` with no tour running is a no-op)
- `--suggest` (updates the Home banner without showing the window)
- `--screenshot` (capture, not a user-visible open)

## Commands honored (`bread.command.help.*`)

These are only received while `breadhelp listen` is running. Publishing a
command with no subscriber is a silent no-op; that is the documented
bread convention, not a breadhelp bug.

| Verb | Data | Effect |
|------|------|--------|
| `unload` | none | Runs `breadhelp --unload`: a hidden resident help window quits to free its memory. Does nothing when no window is up, or when it is on screen. Emits `bread.help.unload.done` / `.failed`. |
| `open` | none | Same as launching BOS Help from the app menu (`breadhelp --page home`): show the help window on Home, handed to an already-running instance if there is one. Emits `bread.help.open.done` / `.failed`. |

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

## Presence file (no breadd needed)

While the help window process is up it keeps
`$XDG_RUNTIME_DIR/breadhelp-window.json` current and removes it on exit:

```json
{ "pid": 1234, "visible": false, "resident": true, "since": 1792574783 }
```

`visible` is whether it is on screen, `resident` whether closing it hides
instead of quitting, and `since` the unix time of the last show or hide. A
memory manager (breadram) can read it, or subscribe to `bread.help.opened`
/ `bread.help.hidden`, and send `bread.command.help.unload` (or run
`breadhelp --unload`) once it has been hidden long enough.

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
