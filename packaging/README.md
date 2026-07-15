Arch packaging
==============

`PKGBUILD` builds and installs `breadhelp` from source.

## Local build

```bash
makepkg -si
```

## Before publishing to [breadway] repo

1. Tag a release on GitHub.
2. Update `pkgver` to match the tag.
3. Update `source` to the release tarball URL.
4. Run `updpkgsums` (or manually set `sha256sums`).

## Runtime dependencies

| Package | Required | Notes |
|---------|----------|-------|
| `gtk4` | yes | UI toolkit |
| `glib2` | yes | always |
| `gtk4-layer-shell` | yes | tour overlay (dim mask + spotlight callout) |
| `snapper` | optional | create-backup one-click fix |
