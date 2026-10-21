# breadhelp

Onboarding and help center for [BOS (Bread Operating System)](https://git.breadway.dev/Breadway/bos): a guide library, a keybind reference read live from `~/.config/hypr/binds.json` (Super + / opens it), Ctrl+K search over guides, keybinds, commands and fixes, a troubleshooter with one-click fixes, a short first-run setup (timezone, network, updates, a first snapshot), and a live guided tour overlay that spotlights the real on-screen bread\* apps (breadbar, breadbox, ...) instead of walking through a static wizard.

Two binaries:

- `breadhelp`: the help window. Tauri (Rust in `app/`, Svelte in `frontend/`), the same stack and look as bos-settings. Closing it quits. "Keep ready in the background" (sidebar, off by default) keeps it loaded and hidden instead: Super + / then paints in about 170 ms instead of about 620 ms, for about 200 MB of memory while closed. A hidden resident window reports itself to memory managers and can be unloaded (see EVENTS.md).
- `breadhelp-tour`: the tour overlay. Native GTK4 + gtk4-layer-shell, in `crates/breadhelp-tour`. Started by the help window; exits when the tour ends.

Both share `crates/breadhelp-core` (content, keybinds, state, setup, fixes).

Split out of the `bos` repo into its own repo so a breadhelp release doesn't require a BOS ISO release, and vice versa. A [bakery](https://git.breadway.dev/Breadway/bread-ecosystem) product (`bakery.toml` + bakery CI) and baked into the BOS ISO, not pacman-packaged.

## Building

```bash
npm --prefix frontend ci && npm --prefix frontend run build
cargo build --release                                              # breadhelp
cargo build --release --manifest-path crates/breadhelp-tour/Cargo.toml   # breadhelp-tour
```

The tour has its own Cargo workspace on purpose; see the root `Cargo.toml`. CI runs the same steps through `ci/build-all.sh` and `ci/test-all.sh`.

For frontend work, `cargo tauri dev` in `app/` (or `npm --prefix frontend run dev` and a debug build) gives live reload.

## Packaging / releasing

See `CONTRIBUTING.md` for the single-trunk + tag model. `main` publishes a bakery **dev** build; a `vX.Y.Z-rc.N` tag publishes **beta**; a `vX.Y.Z` tag publishes **stable**. The leftover `packaging/` tree is not a pacman source; bakery is canonical (see `packaging/README.md`).

## bread event integration

Works the same with or without `breadd`. When the main window is shown,
breadhelp publishes `bread.help.opened`. See [EVENTS.md](EVENTS.md).
