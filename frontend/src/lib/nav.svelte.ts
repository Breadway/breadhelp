// Which page is showing, and which guide. One window, so module state.

import type { Mode, Page } from "./api";
import { api } from "./api";

export const nav = $state({
	page: "keybinds" as Page,
	/** `category/id` of the guide open on the Guides page. */
	guide: "",
	/** Fix a problem: the symptom tree picked from Home or search. */
	symptom: "",
	searchOpen: false,
	mode: "normal" as Mode,
	suggestion: null as string | null,
	/** Result of the last "Take the tour" attempt, shown while/after it runs. */
	tourStatus: "" as string,
});

/**
 * Start the tour without ever leaving the window silent: the backend keeps
 * the window up until the overlay process is actually running, and this
 * reports either "starting" or the reason it did not.
 */
export async function startTour() {
	nav.tourStatus = "Starting the tour...";
	try {
		const r = await api.startTour();
		if (!r.ok) nav.tourStatus = r.message;
	} catch (e) {
		nav.tourStatus = `Couldn't start the tour: ${String(e)}`;
	}
}

export function go(page: Page, opts: { guide?: string; symptom?: string } = {}) {
	nav.page = page;
	if (opts.guide !== undefined) nav.guide = opts.guide;
	if (opts.symptom !== undefined) nav.symptom = opts.symptom;
	nav.searchOpen = false;
}

/** Beginner and Larger text read the simpler guide text where one exists. */
export function prefersSimple(mode: Mode): boolean {
	return mode === "beginner" || mode === "dad";
}
