#!/usr/bin/env python3
"""Check that breadhelp's shared breadbug consent notice matches bread's source.

Usage: scripts/check-consent-sync.py PATH_TO_BREAD_CHECKOUT

breadhelp cannot depend on bread's Rust crate, so the notice is copied into
content/shared/breadbug-consent.md. This script reads CONSENT_TEXT from
bread-bug/src/consent.rs in a bread checkout, applies the Rust string rules
that matter for it (a backslash at the end of a line removes the newline and
the leading whitespace of the next line), and compares the result with the
"## Notice" section of the shared file.

Exit status: 0 when they match, 1 with the first differing line on text drift,
and 2 for a usage or parsing problem. Run it by hand whenever the text
changes in either repository.
"""

import pathlib
import re
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent
SHARED = REPO / "content" / "shared" / "breadbug-consent.md"
CONST = re.compile(r'pub const CONSENT_TEXT: &str = "(.*?)";', re.S)


class UsageError(Exception):
    """The checkout or the shared file could not be read as expected."""


def bread_text(checkout: pathlib.Path) -> str:
    path = checkout / "bread-bug" / "src" / "consent.rs"
    if not path.is_file():
        raise UsageError(f"{path} not found; pass a bread checkout")
    match = CONST.search(path.read_text(encoding="utf-8"))
    if not match:
        raise UsageError(f"CONSENT_TEXT not found in {path}")
    value = re.sub(r"\\\n[ \t]*", "", match.group(1))
    if "\\" in value:
        raise UsageError("unsupported escape in CONSENT_TEXT; extend this script")
    return value


def shared_text() -> str:
    text = SHARED.read_text(encoding="utf-8")
    if "## Notice" not in text:
        raise UsageError(f"no '## Notice' section in {SHARED}")
    return text.split("## Notice", 1)[1].strip()


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__.strip().splitlines()[2], file=sys.stderr)
        return 2
    try:
        want = bread_text(pathlib.Path(sys.argv[1]))
        have = shared_text()
    except UsageError as e:
        print(f"check-consent-sync: {e}", file=sys.stderr)
        return 2
    if want == have:
        print("consent notice matches bread CONSENT_TEXT")
        return 0
    want_lines, have_lines = want.splitlines(), have.splitlines()
    for i in range(max(len(want_lines), len(have_lines))):
        a = want_lines[i] if i < len(want_lines) else "<missing>"
        b = have_lines[i] if i < len(have_lines) else "<missing>"
        if a != b:
            print(
                "consent notice drift: content/shared/breadbug-consent.md differs from "
                f"bread-bug/src/consent.rs at line {i + 1}\n"
                f"  bread:  {a!r}\n  shared: {b!r}\n"
                "Update the shared file and the privacy guide to match bread's constant.",
                file=sys.stderr,
            )
            break
    return 1


if __name__ == "__main__":
    sys.exit(main())
