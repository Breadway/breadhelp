import { test } from "node:test";
import assert from "node:assert/strict";
import { search, score, flat, type Entry } from "./search.ts";

const e = (kind: Entry["kind"], title: string, tags = "", hint = ""): Entry => ({ kind, id: title, title, hint, body: "", tags });

const entries: Entry[] = [
	e("fix", "Reconnect Wi-Fi", "wifi wi-fi network"),
	e("symptom", "Wi-Fi", "wifi"),
	e("guide", "Fixing Wi-Fi problems", "wifi network"),
	e("guide", "Clipboard history", "clipboard"),
	e("keybind", "Clipboard history", "", "Super V"),
	e("command", "breadcrumbs", "wifi profiles"),
];

test("every query word has to match", () => {
	assert.equal(score(e("guide", "Clipboard history"), "clip hist"), 4 + 3);
	assert.equal(score(e("guide", "Clipboard history"), "clip banana"), 0);
});

test("empty query lists everything", () => {
	assert.equal(flat(search(entries, "")).length, entries.length);
});

test("fixes come first and share one group", () => {
	const groups = search(entries, "wifi");
	assert.deepEqual(groups.map((g) => g.title), ["Fixes", "Guides", "Commands"]);
	assert.equal(groups[0].items.length, 2);
});

test("title matches outrank tag matches", () => {
	const items = flat(search([e("guide", "Notes", "wifi"), e("guide", "Wi-Fi setup")], "wi"));
	assert.equal(items[0].title, "Wi-Fi setup");
});

test("regex characters in a query are literal", () => {
	assert.doesNotThrow(() => search(entries, "super + ("));
	assert.equal(flat(search(entries, "(")).length, 0);
});

test("hyphens don't matter", () => {
	assert.ok(score(e("symptom", "Wi-Fi troubleshooter"), "wifi") > 0);
	assert.ok(score(e("guide", "Wifi setup"), "wi-fi") > 0);
});
