// The decision behind every Run control: a fix authored with
// `requires_confirm` asks first, and only runs once the shared dialog has
// been answered. Kept pure so fixConfirm.test.ts can cover the gate and the
// pending/answer transitions without a webview; the Rust side enforces the
// same rule independently.

export type Gate = "ask" | "run";

export interface FixChoice {
	label: string;
	description?: string;
	requiresConfirm: boolean;
}

/** What a Run control should do: ask the person first, or run directly. */
export function gate(fix: { requiresConfirm: boolean }, confirmed: boolean): Gate {
	return fix.requiresConfirm && !confirmed ? "ask" : "run";
}

export interface ConfirmDecision {
	/** False when the person declined: the caller must not invoke a command. */
	proceed: boolean;
	/** True only when a prompt was shown and accepted. This is what the
	 *  backend's `confirmed` argument means, not a copy of the content flag. */
	confirmed: boolean;
}

export interface PendingFix {
	label: string;
	detail: string;
	resolve: (confirmed: boolean) => void;
}

export interface ConfirmationState {
	pending: PendingFix | null;
}

/** Open the prompt for `fix`. A prompt that was already open is cancelled
 *  first, so its caller returns without running anything. */
export function openConfirmation(state: ConfirmationState, fix: { label: string; description?: string }, resolve: (confirmed: boolean) => void) {
	state.pending?.resolve(false);
	state.pending = {
		label: fix.label,
		detail: fix.description ?? "This fix changes this computer. Run it now?",
		resolve,
	};
}

/** Answer the open prompt. Returns whether there was a prompt to answer. */
export function answerPending(state: ConfirmationState, confirmed: boolean): boolean {
	const pending = state.pending;
	state.pending = null;
	pending?.resolve(confirmed);
	return pending !== null;
}
