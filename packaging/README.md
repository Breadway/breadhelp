# packaging/

Leftover from the pacman era. breadhelp is bakery-managed now
(`bakery.toml` plus `.forgejo/workflows/{dev-release,rc-release,release}.yml`)
and is baked into the BOS ISO.

`breadhelp.desktop` is still live: bakery CI copies it into the published
artifact. There is no PKGBUILD and no `package.yml` — do not treat this
directory as a pacman package source.
