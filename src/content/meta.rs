//! `meta.toml` schema: one per guide directory.

#[derive(serde::Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    #[default]
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(serde::Deserialize, Clone, Default)]
pub struct GuideMeta {
    pub title: String,
    /// Not read by any UI yet: reserved for a future difficulty badge.
    /// Real content already sets it, so kept rather than dropped.
    #[allow(dead_code)]
    #[serde(default)]
    pub difficulty: Difficulty,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Guide dir-names in the same or another category. Unresolvable entries
    /// are silently dropped when rendering "related guides": never a crash.
    /// Not read by any UI yet; the "related guides" render path doesn't
    /// exist. Real content already sets it, so kept rather than dropped.
    #[allow(dead_code)]
    #[serde(default)]
    pub related: Vec<String>,
    /// Not read by any UI yet: reserved for a future "N min read" hint.
    /// Real content already sets it, so kept rather than dropped.
    #[allow(dead_code)]
    #[serde(default)]
    pub estimated_minutes: u32,
    /// Overrides the parent directory name as the guide's category, if set.
    #[serde(default)]
    pub category: Option<String>,
}
