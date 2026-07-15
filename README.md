# breadhelp

Onboarding and help center for [BOS (Bread Operating System)](https://github.com/Breadway/bos) — GTK4, a searchable guide library, an interactive keybind viewer, a troubleshooting wizard with one-click fixes, and a live guided tour overlay that spotlights real on-screen bread\* apps (breadbar, breadbox, ...) instead of walking through a static wizard.

Split out of the `bos` repo into its own repo so a breadhelp release doesn't require a BOS ISO release, and vice versa. Pacman-packaged (not bakery-managed) — see `packaging/README.md`.

## Building

```bash
cargo build --release
```

## Packaging / releasing

See `packaging/README.md`. In short: bump `Cargo.toml`'s version, tag `vX.Y.Z`, push the tag to both remotes — `.forgejo/workflows/package.yml` builds and publishes to the `[breadway]` pacman repo automatically.
