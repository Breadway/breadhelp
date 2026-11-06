import { test } from "node:test";
import assert from "node:assert/strict";
import { answerPending, gate, openConfirmation, type ConfirmationState } from "./fixConfirm.ts";

test("a requires_confirm fix asks until the dialog is answered", () => {
	assert.equal(gate({ requiresConfirm: true }, false), "ask");
	assert.equal(gate({ requiresConfirm: true }, true), "run");
});

test("a fix without the flag runs directly", () => {
	assert.equal(gate({ requiresConfirm: false }, false), "run");
	assert.equal(gate({ requiresConfirm: false }, true), "run");
});

test("opening a prompt leaves it unanswered until the person responds", () => {
	const state: ConfirmationState = { pending: null };
	const answers: boolean[] = [];
	openConfirmation(state, { label: "Reset the network", description: "Turns Wi-Fi off and on." }, (ok) => answers.push(ok));

	assert.equal(state.pending?.label, "Reset the network");
	assert.equal(state.pending?.detail, "Turns Wi-Fi off and on.");
	assert.deepEqual(answers, []);

	assert.equal(answerPending(state, false), true);
	assert.equal(state.pending, null);
	assert.deepEqual(answers, [false]);
});

test("accepting resolves true and a second answer does nothing", () => {
	const state: ConfirmationState = { pending: null };
	const answers: boolean[] = [];
	openConfirmation(state, { label: "Update everything" }, (ok) => answers.push(ok));

	assert.equal(answerPending(state, true), true);
	assert.deepEqual(answers, [true]);
	assert.equal(answerPending(state, true), false);
	assert.deepEqual(answers, [true]);
});

test("a new prompt cancels the one that was still open", () => {
	const state: ConfirmationState = { pending: null };
	const first: boolean[] = [];
	const second: boolean[] = [];
	openConfirmation(state, { label: "First" }, (ok) => first.push(ok));
	openConfirmation(state, { label: "Second" }, (ok) => second.push(ok));

	assert.deepEqual(first, [false]);
	assert.equal(state.pending?.label, "Second");
	assert.equal(answerPending(state, true), true);
	assert.deepEqual(second, [true]);
});
