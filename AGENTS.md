# AGENTS.md — Repo hygiene

Scope: this file covers *repo hygiene* — branching, remotes, CI, cleanup. It is not project documentation.

## Identity
- breadhelp is a **bakery** product (`bakery.toml` + bakery CI + BOS ISO bake). It is not pacman-packaged.

## Branch model
- One long-lived branch: **`main`**. There is no separate `dev` or `beta` branch — those are bakery tracks, published from `main` and from tags. Follow `CONTRIBUTING.md`.
- Short-lived work: `feature/<name>` or `fix/<name>` off `main`.

## Remotes
- `origin` — Forgejo on Hestia only (`ssh://git@100.66.238.26:2222/Breadway/breadhelp.git`), via Tailscale. No GitHub mirror, unlike most bread-ecosystem repos.

## CI
- `.forgejo/workflows/dev-release.yml` — push to `main` (bakery dev track).
- `.forgejo/workflows/rc-release.yml` — `vX.Y.Z-rc.N` tags (bakery beta track).
- `.forgejo/workflows/release.yml` — other `v*` tags (signed bakery stable).
- There is no `package.yml` / pacman publish workflow.

## Don't
- Don't embed credentials in remote URLs — this repo already uses SSH, keep it that way.
- Don't start a Tauri port — this stays GTK4.
