//! First-run setup, shown in the help window on the first login (see
//! `breadhelp_core::setup`). Finishing or skipping it hands off to the tour.

use serde::Serialize;
use tauri::AppHandle;

use breadhelp_core::config::State;
use breadhelp_core::services::exec;
use breadhelp_core::{setup, tourctl};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    label: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupState {
    needed: bool,
    step: usize,
    step_count: usize,
    timezone: String,
    timezones: Vec<String>,
    network_tool: Option<Tool>,
    updates_tool: Option<Tool>,
    has_breadcrumbs: bool,
    snapper: bool,
    snapshot_attempted: bool,
}

#[tauri::command]
pub async fn get_setup() -> SetupState {
    tauri::async_runtime::spawn_blocking(|| {
        let state = State::load();
        let (timezone, timezones) = setup::load_timezones();
        let has_settings = exec::command_exists("bos-settings");
        SetupState {
            needed: setup::needs_setup(),
            step: (state.day_zero_step().max(0) as usize).min(setup::STEP_COUNT - 1),
            step_count: setup::STEP_COUNT,
            timezone,
            timezones,
            network_tool: setup::pick_network_tool(has_settings, exec::command_exists("nm-connection-editor"))
                .map(|(label, ..)| Tool { label }),
            updates_tool: setup::pick_updates_tool(has_settings).map(|(label, ..)| Tool { label }),
            has_breadcrumbs: exec::command_exists("breadcrumbs"),
            snapper: setup::snapper_available(),
            snapshot_attempted: state.day_zero_snapshot_attempted(),
        }
    })
    .await
    .expect("setup state thread")
}

#[tauri::command]
pub fn set_setup_step(step: usize) {
    State::load().set_day_zero_step(step.min(setup::STEP_COUNT - 1) as i64);
}

#[derive(Serialize)]
pub struct Outcome {
    message: String,
    ok: bool,
}

#[tauri::command]
pub async fn apply_timezone(tz: String) -> Outcome {
    let (message, ok) = tauri::async_runtime::spawn_blocking(move || setup::apply_timezone(&tz))
        .await
        .unwrap_or_else(|_| ("Couldn't change the timezone.".into(), false));
    Outcome { message, ok }
}

#[tauri::command]
pub fn open_network_tool() {
    let has_settings = exec::command_exists("bos-settings");
    if let Some((_, prog, args)) = setup::pick_network_tool(has_settings, exec::command_exists("nm-connection-editor")) {
        exec::run_argv(prog, args);
    }
}

#[tauri::command]
pub fn open_updates_tool() {
    if let Some((_, prog, args)) = setup::pick_updates_tool(exec::command_exists("bos-settings")) {
        exec::run_argv(prog, args);
    }
}

#[tauri::command]
pub async fn create_snapshot() -> Outcome {
    State::load().set_day_zero_snapshot_attempted(true);
    let ok = tauri::async_runtime::spawn_blocking(setup::create_snapshot).await.unwrap_or(false);
    Outcome { message: setup::snapshot_message(ok).into(), ok }
}

/// Finish (or skip) setup: mark it done, start the tour, and get the window
/// out of the way
/// unless the user already finished it once.
#[tauri::command]
pub fn finish_setup(app: AppHandle) {
    let mut state = State::load();
    state.set_day_zero_completed(true);
    if !state.onboarding_completed() {
        tourctl::spawn_tour(&["--start"]);
    }
    super::window::dismiss(&app);
}
