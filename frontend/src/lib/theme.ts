// Bridges bread-theme's wallpaper palette into the page: fetch once at
// startup, then follow the backend's `theme-changed` pushes (see
// app/src/commands/theme.rs). Same as bos-settings' $lib/theme.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const STYLE_ID = "bread-theme";

function apply(css: string) {
	let style = document.getElementById(STYLE_ID);
	if (!style) {
		style = document.createElement("style");
		style.id = STYLE_ID;
		document.head.appendChild(style);
	}
	style.textContent = css;
}

export async function initTheme(): Promise<void> {
	try {
		apply(await invoke<string>("get_theme_css"));
	} catch {
		// No palette: app.css fallbacks already look like BOS.
	}
	await listen<string>("theme-changed", (event) => apply(event.payload));
}
