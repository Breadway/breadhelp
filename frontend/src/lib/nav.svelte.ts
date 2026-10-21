// Which page is showing, and which guide. One window, so module state.

import type { Mode, Page } from "./api";

export const nav = $state({
	page: "keybinds" as Page,
	/** `category/id` of the guide open on the Guides page. */
	guide: "",
	/** Fix a problem: the symptom tree picked from Home or search. */
	symptom: "",
	searchOpen: false,
	mode: "normal" as Mode,
	suggestion: null as string | null,
});

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
