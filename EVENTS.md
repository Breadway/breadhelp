# breadhelp — bread event integration

breadhelp is a standalone GTK help center: it works exactly the same with
or without `breadd` running. When breadd *is* present, it publishes one
event after the main window is actually shown. See the parent `bread`
repo's `Documentation.md` — specifically its "Namespaces" and
"Integrating a bread\* app" sections — for the general convention this
follows.

App id: **`help`**. Transport: `bread-utils`'s `bread_client` module
(feature `bread-client`) — breadhelp links it directly. Each `emit` is
its own short-lived connection. There is no long-running command
subscription.

## Events published (`bread.help.*`)

| Event | Data | When |
|-------|------|------|
| `bread.help.opened` | `{ "autostart": bool }` | The main help window is presented (`ApplicationWindow::present`). `autostart` is `true` when that invocation was launched with `--autostart`. |

Not emitted when:

- every-login `--autostart` builds a hidden window because onboarding is
  already done (silent autostart)
- first-run `--autostart` starts the tour overlay without presenting the
  main window
- `--onboard` / `--tour-event` (tour only)
- `--screenshot` (capture, not a user-visible open)

## Commands honored (`bread.command.help.*`)

None. Opening the help center, starting the tour, and applying one-click
fixes already exist as local CLI / UI paths. There is no command
subscription, and no verb is stubbed as a no-op. If breadhelp later grows
a bus verb that maps to real behavior, add it then.

## Fail-safe behavior

- If breadd isn't installed or isn't running, `emit` is a silent no-op
  (`BreadClient::emit` never blocks or errors the caller) — the help
  center, tour, and screenshots are entirely unaffected.
- There is no command subscription to reconnect.
