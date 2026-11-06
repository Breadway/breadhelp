//! Regression for HELP-20: the installed binary must find its shipped guides
//! without a source checkout and without relying on a host-specific content
//! directory.
//!
//! This intentionally uses `ContentStore::load`, the same entry point the
//! Tauri commands use. It never scans the repo's `content/` tree through
//! `CARGO_MANIFEST_DIR`; the bytes must come from the installed layout or the
//! embedded fallback compiled into the binary.

use breadhelp_core::content::ContentStore;

#[test]
fn installed_build_lists_and_opens_shipped_guides() {
    let store = ContentStore::load();

    assert!(
        !store.all().is_empty(),
        "installed breadhelp found no guides: check the system content root and the embedded fallback"
    );
    assert!(!store.categories().is_empty());

    for guide in store.all() {
        assert!(
            !guide.meta.title.trim().is_empty(),
            "{}/{} has an empty title",
            guide.category,
            guide.id
        );
        let body = store.body(guide, false);
        assert!(
            !body.trim().is_empty(),
            "{}/{} has no content.md body",
            guide.category,
            guide.id
        );
    }
}
