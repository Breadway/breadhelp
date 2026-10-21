<script lang="ts">
	// A guide's block/span tree as text nodes, keycaps and buttons. Guide text
	// never reaches the page as HTML.
	import Keys from "./Keys.svelte";
	import type { GuideBlock } from "$lib/api";

	let { blocks, onaction }: { blocks: GuideBlock[]; onaction?: (action: number) => void } = $props();

	// Consecutive bullets share one <ul>, so screen readers announce a list.
	type Group = { kind: "list"; items: GuideBlock[] } | { kind: "single"; block: GuideBlock };
	const groups = $derived.by(() => {
		const out: Group[] = [];
		for (const block of blocks) {
			const last = out[out.length - 1];
			if (block.kind === "bullet") {
				if (last && last.kind === "list") last.items.push(block);
				else out.push({ kind: "list", items: [block] });
			} else {
				out.push({ kind: "single", block });
			}
		}
		return out;
	});
</script>

{#snippet spans(block: GuideBlock)}
	{#each block.spans as s, si (si)}
		{#if s.type === "text"}
			{#each s.runs as r, ri (ri)}{#if r.code}<code>{r.text}</code>{:else if r.bold}<b>{r.text}</b>{:else}{r.text}{/if}{/each}
		{:else if s.type === "keys"}
			<Keys caps={s.caps} />
		{:else if onaction}
			<button class="btn primary inline" onclick={() => onaction(s.action)}>{s.label}</button>
		{/if}
	{/each}
{/snippet}

{#each groups as g, gi (gi)}
	{#if g.kind === "list"}
		<ul>
			{#each g.items as item, ii (ii)}<li>{@render spans(item)}</li>{/each}
		</ul>
	{:else if g.block.kind === "heading"}
		<h2>{@render spans(g.block)}</h2>
	{:else}
		<p>{@render spans(g.block)}</p>
	{/if}
{/each}

<style>
	h2 { font-family: var(--font-display, 'Outfit', sans-serif); font-size: 17px; margin: 22px 0 10px; }
	ul { margin: 0 0 12px; padding-left: 18px; }
	p, li { line-height: 1.6; color: color-mix(in srgb, var(--fg) 86%, transparent); margin: 0 0 12px; }
	li { margin-bottom: 8px; }
	code { font-family: ui-monospace, monospace; font-size: 0.92em; background: var(--surface-2); border-radius: 5px; padding: 1px 5px; }
	.inline { min-height: 32px; padding: 5px 12px; margin: 2px 0; }
</style>
