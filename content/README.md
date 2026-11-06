# breadhelp content

Guides and troubleshooting data. The runtime looks in this order:

1. `~/.local/share/breadhelp/content/` (user overrides/additions)
2. `/usr/local/share/breadhelp/content/` (bakery system install on BOS)
3. `/usr/share/breadhelp/content/` (manual or distro system install)
4. This `content/` tree embedded in the binaries as a fallback

A colliding guide directory name wins wholesale over a lower-precedence copy.
Bakery ships this tree as `content.tar.gz`; BOS extracts it under
`/usr/local/share/breadhelp/` at bake time.

Categories are just top-level directories here: the Guides page lists whatever
exists, so adding a category is a content change, not a code change:

```
getting-started/
daily-use/
customization/
apps/
troubleshooting/
  _symptoms/          # troubleshooting wizard's symptom-tree *.toml files
advanced/
```

Each guide is a directory with:

- `meta.toml`: title, difficulty, tags, related guides, estimated time
- `content.md`: the guide body (constrained Markdown subset + `[Run]`/
  `[Terminal]`/`[Show Keybind]` tags, see
  `crates/breadhelp-core/src/content/markdown.rs`). Lines wrap freely: a
  paragraph or bullet runs until a blank line, a heading or the next `- `.
  `[Show Keybind]` takes what a person types (`super + ,`) and finds the
  bind stored under its keysym name (`comma`); it renders as keycaps.
  `[Run]` goes through a shell; `[Terminal]` splits on whitespace and opens
  the BOS terminal with no shell, and is the only tag for terminal-only
  commands such as `breadbug`.
- `content.beginner.md`: optional, the "Simpler" version. Beginner and
  Larger text display modes open it by default; the guide page has a
  Full / Simpler switch whenever it exists.

`troubleshooting/_symptoms/*.toml` are the Fix a problem trees: top-level
`title` and `summary` (the card), then `[[node]]` entries with a `prompt`,
`[[node.options]]` (`label`, `goto` another node id) or a `[node.fix]`
(`description`, `command`, optional `requires_confirm` to ask first).

`tours/onboarding.toml` is the guided tour. Each `[[step]]` has `title` and
`body`, optionally a `target_namespace` / `target_window_class` to
spotlight, `launch` (run only by the "Show me" button), `success_event`
(makes it a must-do step), `keybind` (keycaps on its "waiting for you"
row), `confirmed_text`, and `[[step.shortcuts]]` (`label`, `keybind`) for a
keycap list.
