//! Everything breadhelp knows that isn't UI: guide content, keybinds,
//! saved state, the setup steps and the system calls behind fixes. Shared by
//! the help window (`breadhelp`, Tauri) and the tour overlay
//! (`breadhelp-tour`, GTK4).

pub mod config;
pub mod content;
pub mod fixes;
pub mod services;
pub mod setup;
pub mod tourctl;
