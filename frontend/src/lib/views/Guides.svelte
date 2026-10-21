<script lang="ts">
	// Guide reader: categories down the left, the guide on the right. Guide
	// text arrives as a block/span tree (never HTML) and renders as text
	// nodes, keycaps and buttons.
	import GuideBlocks from "$lib/components/GuideBlocks.svelte";
	import { api, type Guide, type GuideSummary } from "$lib/api";
	import { data } from "$lib/data.svelte";
	import { go, nav, prefersSimple } from "$lib/nav.svelte";

	let guide: Guide | null = $state(null);
	let simple = $state(prefersSimple(nav.mode));
	let article: HTMLElement | undefined = $state();

	const categories = $derived.by(() => {
		const out: { title: string; items: GuideSummary[] }[] = [];
		for (const g of data.guides) {
			const last = out[out.length - 1];
			if (last && last.title === g.categoryTitle) last.items.push(g);
			else out.push({ title: g.categoryTitle, items: [g] });
		}
		return out;
	});
	const currentKey = $derived(nav.guide || (data.guides[0] ? `${data.guides[0].category}/${data.guides[0].id}` : ""));
	const current = $derived(data.guides.find((g) => `${g.category}/${g.id}` === currentKey));
	const next = $derived.by(() => {
		const i = data.guides.findIndex((g) => g === current);
		return i >= 0 ? data.guides[(i + 1) % data.guides.length] : undefined;
	});

	// A new guide opens in the reading level the display mode prefers.
	$effect(() => {
		void currentKey;
		simple = prefersSimple(nav.mode);
	});

	$effect(() => {
		const g = current;
		const s = simple;
		if (!g) return;
		api.guide(g.category, g.id, s).then((v) => {
			guide = v;
			if (article) article.scrollTop = 0;
		});
	});

	function runAction(action: number) {
		if (current) api.runGuideAction(current.category, current.id, guide?.simplified ?? false, action);
	}
</script>

<div class="reader">
	<nav class="list" aria-label="Guides">
		{#each categories as c (c.title)}
			<div class="cat">{c.title}</div>
			{#each c.items as g (g.id)}
				{@const key = `${g.category}/${g.id}`}
				<button class="g" class:on={key === currentKey} aria-current={key === currentKey ? "page" : undefined} onclick={() => go("guides", { guide: key })}>{g.title}</button>
			{/each}
		{/each}
	</nav>

	<article class="article" bind:this={article}>
		{#if guide && current}
			<div class="col">
				<div class="meta">
					<span>{guide.categoryTitle}</span>
					{#if current.hasSimple}
						<div class="seg" role="group" aria-label="Reading level">
							<button aria-pressed={!simple} class:on={!simple} onclick={() => (simple = false)}>Full</button>
							<button aria-pressed={simple} class:on={simple} onclick={() => (simple = true)}>Simpler</button>
						</div>
					{/if}
				</div>
				<h1 class="title">{guide.title}</h1>
				<GuideBlocks blocks={guide.blocks} onaction={runAction} />
				<footer class="foot">
					<span>Works offline. Guides update with bakery.</span>
					{#if next && next !== current}
						<button class="next" onclick={() => go("guides", { guide: `${next.category}/${next.id}` })}>Next: {next.title}</button>
					{/if}
				</footer>
			</div>
		{:else if data.loaded && data.guides.length === 0}
			<p class="none">No guides are installed. They ship with breadhelp's content package.</p>
		{/if}
	</article>
</div>

<style>
	.reader { display: flex; min-height: 100%; }
	.list { flex: 0 0 220px; padding: 18px 10px; border-right: 1px solid var(--line); overflow-y: auto; }
	.cat { font-size: 10.5px; font-weight: 500; letter-spacing: 0.1em; text-transform: uppercase; color: var(--muted); padding: 12px 10px 4px; }
	.g { display: block; width: 100%; min-height: 32px; padding: 7px 10px; border: 0; border-radius: 9px; background: transparent; color: var(--fg); text-align: left; font-size: 13px; cursor: pointer; }
	.g:hover { background: color-mix(in srgb, var(--fg) 4%, transparent); }
	.g.on { background: var(--accent-soft); }
	.article { flex: 1; min-width: 0; padding: 28px 36px 48px; overflow-y: auto; }
	.col { max-width: 620px; }
	.meta { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; font-size: 12px; color: var(--muted); }
	.meta span { flex: 1 1 160px; }
	.seg { display: inline-flex; background: var(--surface-2); border-radius: 10px; padding: 3px; gap: 2px; }
	.seg button { padding: 6px 11px; border-radius: 8px; border: 0; background: none; color: var(--muted); font-size: 12.5px; cursor: pointer; }
	.seg button.on { background: var(--accent-soft); color: var(--fg); box-shadow: inset 0 0 0 1px var(--accent-edge); }
	.title { margin: 6px 0 16px; font-size: 28px; }
	.foot { margin-top: 28px; padding-top: 14px; border-top: 1px solid var(--line); display: flex; justify-content: space-between; gap: 12px; flex-wrap: wrap; font-size: 12.5px; color: var(--muted); }
	.next { background: none; border: 0; padding: 0; color: var(--accent); cursor: pointer; font-size: 12.5px; }
	.none { color: var(--muted); }
	@media (max-width: 760px) {
		.reader { flex-direction: column; }
		.list { flex: none; border-right: 0; border-bottom: 1px solid var(--line); max-height: 200px; }
		.article { padding: 20px 16px 40px; }
	}
</style>
