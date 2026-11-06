// Bridges bread-theme's wallpaper palette into the page: fetch once at
// startup, then follow the backend's `theme-changed` pushes (see
// app/src/commands/theme.rs). Same as bos-settings' $lib/theme.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { inkOn } from "./contrast";

const STYLE_ID = "bread-theme";

/**
 * Re-derive --on-accent from the actual accent the theme resolved to. An
 * older palette can omit it, and a stale one can pair dark ink with a dark
 * accent; either way the primary buttons in Setup are unreadable.
 */
function applyAccentInk() {
	const probe = document.createElement("span");
	probe.style.cssText = "position:absolute;visibility:hidden;color:var(--accent)";
	document.body.appendChild(probe);
	const accent = getComputedStyle(probe).color;
	probe.remove();
	if (!accent) return;
	document.documentElement.style.setProperty("--on-accent", inkOn(accent));
}

function apply(css: string) {
	let style = document.getElementById(STYLE_ID);
	if (!style) {
		style = document.createElement("style");
		style.id = STYLE_ID;
		document.head.appendChild(style);
	}
	style.textContent = css;
	applyAccentInk();
}

export async function initTheme(): Promise<void> {
	try {
		apply(await invoke<string>("get_theme_css"));
	} catch {
		// No palette: app.css fallbacks already look like BOS.
	}
	await listen<string>("theme-changed", (event) => apply(event.payload));
}
