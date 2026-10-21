# AGENTS.md: Repo hygiene

Scope: this file covers *repo hygiene*: branching, remotes, CI, cleanup. It is not project documentation.

## Identity
- breadhelp is a **bakery** product (`bakery.toml` + bakery CI + BOS ISO bake). It is not pacman-packaged.

## Branch model
- One long-lived branch: **`main`**. There is no separate `dev` or `beta` branch; those are bakery tracks, published from `main` and from tags. Follow `CONTRIBUTING.md`.
- Short-lived work: `feature/<name>` or `fix/<name>` off `main`.

## Remotes
- `origin`: Forgejo on Hestia only (`ssh://git@100.66.238.26:2222/Breadway/breadhelp.git`), via Tailscale. No GitHub mirror, unlike most bread-ecosystem repos.

## CI
- `.forgejo/workflows/dev-release.yml`: push to `main` (bakery dev track).
- `.forgejo/workflows/rc-release.yml`: `vX.Y.Z-rc.N` tags (bakery beta track).
- `.forgejo/workflows/release.yml`: other `v*` tags (signed bakery stable).
- `.forgejo/workflows/check.yml`: `feature/**` / `fix/**` pushes.
- Every workflow builds through `ci/build.sh` with `ci/build-all.sh` / `ci/test-all.sh`: the Svelte frontend plus two Cargo workspaces (repo root, and `crates/breadhelp-tour`).
- There is no `package.yml` / pacman publish workflow.

## Stack
- Two binaries. `breadhelp` (the help window) is Tauri + Svelte, like bos-settings (decided in HELP-17, 2026-10-21). `breadhelp-tour` (the guided tour overlay) stays native GTK4 + gtk4-layer-shell.
- Keep them in separate Cargo workspaces: in one build, Cargo unifies bread-theme's `gtk` feature into the Tauri binary, which then links GTK4 next to GTK3 and aborts at start.

## Don't
- Don't embed credentials in remote URLs; this repo already uses SSH, keep it that way.
- Don't move the tour overlay into the Tauri binary: layer-shell needs GTK4, and GTK3 and GTK4 can't share a process.
