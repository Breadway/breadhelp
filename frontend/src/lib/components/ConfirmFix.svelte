<script lang="ts">
	// The single confirmation prompt for requires_confirm fixes. Mounted once
	// by the page; the Fix view, FixList and SearchPalette all open it with
	// confirmFix() rather than drawing their own dialog.
	import { onMount } from "svelte";
	import { answerConfirmation, confirmation } from "$lib/confirm.svelte";

	let dialog: HTMLDivElement | undefined = $state();

	onMount(() => dialog?.focus());

	function onKey(e: KeyboardEvent) {
		if (e.key === "Escape") {
			e.preventDefault();
			e.stopPropagation();
			answerConfirmation(false);
		}
	}
</script>

{#if confirmation.pending}
	<div
		class="backdrop"
		role="presentation"
		onclick={(e) => {
			if (e.target === e.currentTarget) answerConfirmation(false);
		}}
	>
		<div class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="confirm-title" tabindex="-1" bind:this={dialog} onkeydown={onKey}>
			<h2 id="confirm-title">Run "{confirmation.pending.label}"?</h2>
			<p>{confirmation.pending.detail}</p>
			<div class="acts">
				<button class="btn" onclick={() => answerConfirmation(false)}>Cancel</button>
				<button class="btn primary" onclick={() => answerConfirmation(true)}>Yes, run it</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.backdrop { position: fixed; inset: 0; z-index: 40; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; padding: 24px; }
	.dialog { width: 100%; max-width: 420px; padding: 20px; border-radius: var(--radius); background: color-mix(in srgb, var(--surface) 88%, var(--bg)); border: 1px solid var(--card-line); box-shadow: 0 24px 60px rgba(0, 0, 0, 0.5); outline: none; }
	.dialog h2 { font-family: var(--font-display, 'Outfit', sans-serif); font-size: 18px; font-weight: 500; line-height: 1.3; }
	.dialog p { margin: 8px 0 18px; color: color-mix(in srgb, var(--fg) 78%, transparent); line-height: 1.5; }
	.acts { display: flex; justify-content: flex-end; gap: 10px; }
</style>
