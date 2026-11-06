// One confirmation prompt, shared by every place a fix can be run: the Fix
// view, the one-click list and Ctrl+K search. The backend refuses a
// requires_confirm fix unless the caller passes confirmed, so this is the
// only route that sets it.

import { gate } from "./fixConfirm";

interface Pending {
	label: string;
	detail: string;
	resolve: (confirmed: boolean) => void;
}

export const confirmation = $state<{ pending: Pending | null }>({ pending: null });

/** Resolves true when the fix may run. A fix without `requires_confirm`
 *  resolves immediately; one with it resolves only after the person accepts
 *  the shared dialog, and false if they cancel. */
export function confirmFix(fix: { label: string; description?: string; requiresConfirm: boolean }): Promise<boolean> {
	if (gate(fix, false) === "run") return Promise.resolve(true);
	return new Promise((resolve) => {
		// A second request replaces the first: the abandoned one must not run.
		confirmation.pending?.resolve(false);
		confirmation.pending = {
			label: fix.label,
			detail: fix.description ?? "This fix changes this computer. Run it now?",
			resolve,
		};
	});
}

/** Answer the open prompt. Cancelling resolves false, so the caller returns
 *  before it ever invokes the command. */
export function answerConfirmation(confirmed: boolean) {
	const pending = confirmation.pending;
	confirmation.pending = null;
	pending?.resolve(confirmed);
}
