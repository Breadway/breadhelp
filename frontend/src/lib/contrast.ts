// Text ink for an accent fill. The shared bread-theme palette already emits
// --on-accent, but an older palette or a hand-written theme can leave it
// missing or stale, which is how dark text ended up on a dark blue button.
// Picking the better of the two ecosystem inks here makes contrast a
// property of this window, not of whichever theme happened to be installed.

const LIGHT_INK = "#f5f5f5";
const DARK_INK = "#11111b";

function channel(v: number): number {
	const c = v / 255;
	return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

/** sRGB relative luminance (WCAG), or null for a colour we cannot parse. */
export function relativeLuminance(colour: string): number | null {
	const c = colour.trim().toLowerCase();
	let r: number;
	let g: number;
	let b: number;
	if (c.startsWith("#")) {
		let hex = c.slice(1);
		if (hex.length === 3) hex = hex.split("").map((x) => x + x).join("");
		if (!/^[0-9a-f]{6}$/.test(hex)) return null;
		r = parseInt(hex.slice(0, 2), 16);
		g = parseInt(hex.slice(2, 4), 16);
		b = parseInt(hex.slice(4, 6), 16);
	} else {
		const m = c.match(/^rgba?\(\s*([\d.]+)[,\s]+([\d.]+)[,\s]+([\d.]+)/);
		if (!m) return null;
		r = Number(m[1]);
		g = Number(m[2]);
		b = Number(m[3]);
		if (![r, g, b].every((v) => Number.isFinite(v) && v >= 0 && v <= 255)) return null;
	}
	return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

/** WCAG contrast ratio between two relative luminances. */
export function contrastRatio(a: number, b: number): number {
	const hi = Math.max(a, b);
	const lo = Math.min(a, b);
	return (hi + 0.05) / (lo + 0.05);
}

/**
 * A legible ink for text on `colour`: whichever of the ecosystem's near-black
 * or near-white wins. Falls back to light ink when the colour is unusable,
 * since a dark UI is the common case.
 */
export function inkOn(colour: string): string {
	const l = relativeLuminance(colour);
	if (l === null) return LIGHT_INK;
	const light = relativeLuminance(LIGHT_INK) as number;
	const dark = relativeLuminance(DARK_INK) as number;
	return contrastRatio(l, light) >= contrastRatio(l, dark) ? LIGHT_INK : DARK_INK;
}

/** The ratio `inkOn` guarantees for a colour (used by tests). */
export function inkContrast(colour: string): number {
	const l = relativeLuminance(colour);
	if (l === null) return 0;
	return contrastRatio(l, relativeLuminance(inkOn(colour)) as number);
}
