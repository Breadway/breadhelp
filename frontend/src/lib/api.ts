// Typed wrappers over the backend's Tauri commands (app/src/commands/).
// Shapes mirror the Rust structs' camelCase serde output.

import { invoke } from "@tauri-apps/api/core";

export type Page = "home" | "keybinds" | "guides" | "fix" | "setup";
export type Mode = "normal" | "beginner" | "dad" | "compact";

export interface GuideSummary {
	category: string;
	categoryTitle: string;
	id: string;
	title: string;
	tags: string[];
	hasSimple: boolean;
}

export interface TextRun { text: string; bold: boolean; code: boolean }
export type GuideSpan =
	| { type: "text"; runs: TextRun[] }
	| { type: "action"; label: string; action: number }
	| { type: "keys"; caps: string[] };
export interface GuideBlock { kind: "heading" | "paragraph" | "bullet"; spans: GuideSpan[] }
export interface Guide { title: string; categoryTitle: string; simplified: boolean; blocks: GuideBlock[] }

export interface KeyRow {
	caps: string[];
	label: string;
	category: string;
	comboId: string | null;
	command: string | null;
	canTry: boolean;
	canRebind: boolean;
}
export interface Keybinds { rows: KeyRow[]; categories: { id: string; title: string }[]; source: string; bindCount: number }

export interface Fix { id: string; label: string; description: string; tags: string; requiresConfirm: boolean }
export interface GalleryCommand { name: string; description: string }

export interface SymptomNode { id: string; prompt: string; options: { label: string; goto: string }[]; fix: string | null; requiresConfirm: boolean }
export interface SymptomTree { id: string; title: string; summary: string; nodes: SymptomNode[] }

export interface Prefs { mode: Mode; hasSettings: boolean; setupNeeded: boolean; setupStepsLeft: number }
export interface Open { page: Page; suggestion: string | null }

export interface Setup {
	needed: boolean;
	step: number;
	stepCount: number;
	timezone: string;
	timezones: string[];
	networkTool: { label: string } | null;
	updatesTool: { label: string } | null;
	hasBreadcrumbs: boolean;
	snapper: boolean;
	snapshotAttempted: boolean;
}
export interface Outcome { message: string; ok: boolean }

export const api = {
	openState: () => invoke<Open>("get_open_state"),
	dismissSuggestion: () => invoke<void>("dismiss_suggestion"),
	hideWindow: () => invoke<void>("hide_window"),
	resident: () => invoke<boolean>("get_resident"),
	setResident: (resident: boolean) => invoke<void>("set_resident", { resident }),
	prefs: () => invoke<Prefs>("get_prefs"),
	setMode: (mode: Mode) => invoke<void>("set_mode", { mode }),

	guides: () => invoke<GuideSummary[]>("list_guides"),
	guide: (category: string, id: string, simplified: boolean) => invoke<Guide | null>("get_guide", { category, id, simplified }),
	runGuideAction: (category: string, id: string, simplified: boolean, action: number) =>
		invoke<void>("run_guide_action", { category, id, simplified, action }),

	keybinds: () => invoke<Keybinds>("get_keybinds"),
	tryKeybind: (comboId: string) => invoke<void>("try_keybind", { comboId }),
	rebindKeybind: (comboId: string, newCombo: string) => invoke<void>("rebind_keybind", { comboId, newCombo }),

	fixes: () => invoke<Fix[]>("list_fixes"),
	runFix: (id: string, confirmed: boolean) => invoke<boolean>("run_fix", { id, confirmed }),
	symptoms: () => invoke<SymptomTree[]>("list_symptoms"),
	runSymptomFix: (tree: string, node: string, confirmed: boolean) => invoke<boolean>("run_symptom_fix", { tree, node, confirmed }),
	commands: () => invoke<GalleryCommand[]>("list_commands"),
	runCommand: (name: string) => invoke<void>("run_gallery_command", { name }),

	openSettings: (page: string) => invoke<void>("open_settings", { page }),
	reportProblem: () => invoke<void>("report_problem"),
	startTour: () => invoke<void>("start_tour"),

	setup: () => invoke<Setup>("get_setup"),
	setSetupStep: (step: number) => invoke<void>("set_setup_step", { step }),
	applyTimezone: (tz: string) => invoke<Outcome>("apply_timezone", { tz }),
	openNetworkTool: () => invoke<void>("open_network_tool"),
	openUpdatesTool: () => invoke<void>("open_updates_tool"),
	createSnapshot: () => invoke<Outcome>("create_snapshot"),
	finishSetup: () => invoke<void>("finish_setup"),
};
