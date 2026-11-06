// The decision behind every Run control: a fix authored with
// `requires_confirm` asks first, and only runs once the shared dialog has
// been answered. Kept pure so fixConfirm.test.ts can cover it without a
// webview; the Rust side enforces the same rule independently.

export type Gate = "ask" | "run";

export function gate(fix: { requiresConfirm: boolean }, confirmed: boolean): Gate {
	return fix.requiresConfirm && !confirmed ? "ask" : "run";
}
