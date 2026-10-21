// The Ctrl+K search over everything the window knows: fixes, symptom
// trees, guides, keybinds and the command gallery. Pure functions, so
// search.test.ts can cover the ranking without a webview.

export type Kind = "fix" | "symptom" | "guide" | "keybind" | "command";

export interface Entry {
	kind: Kind;
	/** Unique within its kind: fix id, tree id, `category/id`, combo id or row label, command name. */
	id: string;
	title: string;
	/** Short right-aligned context ("Getting started", "Super Space"). */
	hint: string;
	/** Preview text. */
	body: string;
	/** Extra words that match but aren't shown. */
	tags: string;
	caps?: string[];
}

export const GROUPS: { kind: Kind; title: string }[] = [
	{ kind: "fix", title: "Fixes" },
	{ kind: "symptom", title: "Fixes" },
	{ kind: "guide", title: "Guides" },
	{ kind: "keybind", title: "Keybinds" },
	{ kind: "command", title: "Commands" },
];

/** Lowercase, and hyphens dropped, so "wifi" finds "Wi-Fi". */
function norm(s: string): string {
	return s.toLowerCase().replace(/-/g, "");
}

function words(q: string): string[] {
	return norm(q).split(/\s+/).filter(Boolean);
}

/** 0 = no match; higher ranks first. Every query word must appear. */
export function score(entry: Entry, query: string): number {
	const qs = words(query);
	if (qs.length === 0) return 1;
	const title = norm(entry.title);
	const hay = norm(`${entry.title} ${entry.hint} ${entry.tags} ${entry.body}`);
	let total = 0;
	for (const q of qs) {
		if (title.startsWith(q)) total += 4;
		else if (new RegExp(`\\b${q.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`).test(title)) total += 3;
		else if (title.includes(q)) total += 2;
		else if (hay.includes(q)) total += 1;
		else return 0;
	}
	return total;
}

export interface Group {
	title: string;
	items: Entry[];
}

/** Matching entries grouped in GROUPS order (fixes first: someone searching
 *  "wifi" most likely wants it working again), best match first in each,
 *  at most `perGroup` each. */
export function search(entries: Entry[], query: string, perGroup = 6): Group[] {
	const out: Group[] = [];
	for (const { kind, title } of GROUPS) {
		const items = entries
			.map((e) => ({ e, s: e.kind === kind ? score(e, query) : 0 }))
			.filter((x) => x.s > 0)
			.sort((a, b) => b.s - a.s || a.e.title.localeCompare(b.e.title))
			.slice(0, perGroup)
			.map((x) => x.e);
		if (items.length === 0) continue;
		const existing = out.find((g) => g.title === title);
		if (existing) existing.items.push(...items);
		else out.push({ title, items });
	}
	return out;
}

/** Flattened order, for arrow-key movement. */
export function flat(groups: Group[]): Entry[] {
	return groups.flatMap((g) => g.items);
}
