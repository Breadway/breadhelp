import { test } from "node:test";
import assert from "node:assert/strict";
import { contrastRatio, inkContrast, inkOn, relativeLuminance } from "./contrast.ts";

test("dark blue accent gets light ink", () => {
	assert.equal(inkOn("#20406f"), "#f5f5f5");
	assert.equal(inkOn("rgb(32, 64, 111)"), "#f5f5f5");
});

test("light tan accent gets dark ink", () => {
	assert.equal(inkOn("#ce9c65"), "#11111b");
});

test("pure black and white resolve to the opposite ink", () => {
	assert.equal(inkOn("#000000"), "#f5f5f5");
	assert.equal(inkOn("#ffffff"), "#11111b");
});

test("short hex is expanded", () => {
	assert.equal(inkOn("#26c"), inkOn("#2266cc"));
});

test("every accent picked by the palette clears WCAG AA", () => {
	for (const colour of ["#20406f", "#ce9c65", "#13345a", "#88aacc", "#444444", "#dddddd"]) {
		assert.ok(inkContrast(colour) >= 4.5, `${colour} only reached ${inkContrast(colour)}`);
	}
});

test("the dark-blue button pair clears WCAG AA for body text", () => {
	// 16,21,34 on 32,64,111: the exact pair the old unthemed button used.
	const ratio = contrastRatio(relativeLuminance("#101522") as number, relativeLuminance("#20406f") as number);
	assert.ok(ratio < 2, "sanity: the old dark-on-dark pair really was low contrast");
	assert.ok(inkContrast("#20406f") >= 4.5);
});

test("unparseable colours fall back instead of throwing", () => {
	assert.equal(inkOn("not-a-colour"), "#f5f5f5");
	assert.equal(relativeLuminance("color-mix(in oklab, red, blue)"), null);
});
