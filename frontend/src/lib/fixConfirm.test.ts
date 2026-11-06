import { test } from "node:test";
import assert from "node:assert/strict";
import { gate } from "./fixConfirm.ts";

test("a requires_confirm fix asks until the dialog is answered", () => {
	assert.equal(gate({ requiresConfirm: true }, false), "ask");
	assert.equal(gate({ requiresConfirm: true }, true), "run");
});

test("a fix without the flag runs directly", () => {
	assert.equal(gate({ requiresConfirm: false }, false), "run");
	assert.equal(gate({ requiresConfirm: false }, true), "run");
});
