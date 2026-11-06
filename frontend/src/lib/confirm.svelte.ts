// One confirmation prompt, shared by every place a fix can be run: the Fix
// view, the one-click list and Ctrl+K search. The backend refuses a
// requires_confirm fix unless the caller passes confirmed, so this is the
// only route that sets it.

import { answerPending, gate, openConfirmation, type ConfirmDecision, type ConfirmationState, type FixChoice } from "./fixConfirm";

export const confirmation = $state<ConfirmationState>({ pending: null });

// Where focus was before the dialog opened, so cancelling returns the person
// to the Run button or search box they came from.
let returnFocus: HTMLElement | null = null;

/** Resolve with `confirmed: true` only after the person accepts the shared
 *  dialog. A fix without `requires_confirm` needs no prompt: proceed, but do
 *  not claim a confirmation that never happened. */
export function confirmFix(fix: FixChoice): Promise<ConfirmDecision> {
	if (gate(fix, false) === "run") return Promise.resolve({ proceed: true, confirmed: false });
	if (typeof document !== "undefined" && document.activeElement instanceof HTMLElement) {
		returnFocus = document.activeElement;
	}
	return new Promise((resolve) => {
		openConfirmation(confirmation, fix, (confirmed) => resolve({ proceed: confirmed, confirmed }));
	});
}

/** Answer the open prompt. Declining resolves `proceed: false`, so the caller
 *  returns before it ever invokes the command. */
export function answerConfirmation(confirmed: boolean) {
	if (answerPending(confirmation, confirmed)) returnFocus?.focus();
	returnFocus = null;
}
