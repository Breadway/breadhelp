# Shared snippets (single source inside breadhelp)

Files here are quoted by guides, not guides themselves: this directory holds
no `meta.toml`, so the content scanner skips it. A test in
`src/content/mod.rs` asserts the privacy guide still quotes the notice below
word for word, so the two breadbug guides cannot drift apart while the
cross-repo sharing is pending.
