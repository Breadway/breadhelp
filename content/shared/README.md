# Shared snippets (single source inside breadhelp)

Files here are quoted by guides, not guides themselves: this directory holds
no `meta.toml`, so the content scanner skips it. A test in
`crates/breadhelp-core/src/content/mod.rs` asserts the privacy guide still
quotes the notice below word for word. bread's `CONSENT_TEXT` is the source
the reporter and bos-settings read; `scripts/check-consent-sync.py` checks
this copy against it. The BOS site still has to publish the same text.
