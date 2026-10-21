<script lang="ts">
	// A breadd suggestion ("New monitor detected: ..."), shown above whichever
	// page is open so it is seen even when Super + / lands on Keybinds.
	import Monitor from "@lucide/svelte/icons/monitor";
	import X from "@lucide/svelte/icons/x";
	import { api } from "$lib/api";
	import { data } from "$lib/data.svelte";
	import { nav } from "$lib/nav.svelte";

	function dismiss() {
		nav.suggestion = null;
		api.dismissSuggestion();
	}
</script>

{#if nav.suggestion}
	<div class="wrap"><div class="banner" role="status">
		<Monitor size={18} />
		<span>{nav.suggestion}</span>
		{#if data.hasSettings}<button class="btn primary" onclick={() => api.openSettings("displays")}>Set up</button>{/if}
		<button class="btn ghost icon" aria-label="Dismiss" onclick={dismiss}><X size={16} /></button>
	</div></div>
{/if}

<style>
	/* Same horizontal frame as the pages' centered column. */
	.wrap { padding: 16px 32px 0; }
	.banner { display: flex; align-items: center; gap: 12px; max-width: 780px; margin: 0 auto; padding: 8px 10px 8px 14px; border-radius: var(--radius); background: var(--accent-soft); border: 1px solid var(--accent-edge); }
	.banner > :global(svg) { color: var(--accent); flex: none; }
	.banner span { flex: 1; min-width: 0; }
	.icon { padding: 7px; min-width: var(--target); }
	@media (max-width: 640px) { .wrap { padding: 12px 16px 0; } }
</style>
