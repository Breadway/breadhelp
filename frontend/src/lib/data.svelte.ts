// Everything the pages list, refetched each time the window opens so edits
// to binds.json or guide files since the last open show up.

import { api, type Fix, type GalleryCommand, type GuideSummary, type Keybinds, type SymptomTree } from "./api";
import type { Entry } from "./search";

export const data = $state({
	guides: [] as GuideSummary[],
	keybinds: null as Keybinds | null,
	fixes: [] as Fix[],
	symptoms: [] as SymptomTree[],
	commands: [] as GalleryCommand[],
	hasSettings: false,
	setupNeeded: false,
	setupStepsLeft: 0,
	loaded: false,
});

export async function refresh(): Promise<void> {
	const [guides, keybinds, fixes, symptoms, commands, prefs] = await Promise.all([
		api.guides(),
		api.keybinds(),
		api.fixes(),
		api.symptoms(),
		api.commands(),
		api.prefs(),
	]);
	Object.assign(data, { guides, keybinds, fixes, symptoms, commands, hasSettings: prefs.hasSettings, setupNeeded: prefs.setupNeeded, setupStepsLeft: prefs.setupStepsLeft, loaded: true });
}

/** The search index, built from the lists above. */
export function searchEntries(): Entry[] {
	const out: Entry[] = [];
	for (const f of data.fixes) {
		out.push({ kind: "fix", id: f.id, title: f.label, hint: "One click", body: f.description, tags: f.tags });
	}
	for (const t of data.symptoms) {
		out.push({ kind: "symptom", id: t.id, title: `${t.title} troubleshooter`, hint: "Step by step", body: t.summary, tags: t.nodes.map((n) => n.prompt).join(" ") });
	}
	for (const g of data.guides) {
		out.push({ kind: "guide", id: `${g.category}/${g.id}`, title: g.title, hint: g.categoryTitle, body: "", tags: g.tags.join(" ") });
	}
	for (const r of data.keybinds?.rows ?? []) {
		out.push({ kind: "keybind", id: r.comboId ?? r.label, title: r.label, hint: r.caps.join(" "), body: r.command ? `Runs ${r.command}` : "", tags: r.category, caps: r.caps });
	}
	for (const c of data.commands) {
		out.push({ kind: "command", id: c.name, title: c.name, hint: "Command", body: c.description, tags: c.description });
	}
	return out;
}
