# breadhelp

Onboarding and help center for [BOS (Bread Operating System)](https://git.breadway.dev/Breadway/bos) — GTK4, a searchable guide library, an interactive keybind viewer, a troubleshooting wizard with one-click fixes, and a live guided tour overlay that spotlights real on-screen bread\* apps (breadbar, breadbox, ...) instead of walking through a static wizard.

Split out of the `bos` repo into its own repo so a breadhelp release doesn't require a BOS ISO release, and vice versa. A [bakery](https://git.breadway.dev/Breadway/bread-ecosystem) product (`bakery.toml` + bakery CI) and baked into the BOS ISO — not pacman-packaged.

## Building

```bash
cargo build --release
```

## Packaging / releasing

See `CONTRIBUTING.md` for the single-trunk + tag model. `main` publishes a bakery **dev** build; a `vX.Y.Z-rc.N` tag publishes **beta**; a `vX.Y.Z` tag publishes **stable**. The leftover `packaging/` tree is not a pacman source — bakery is canonical (see `packaging/README.md`).
