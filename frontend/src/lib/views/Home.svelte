<script lang="ts">
	import Search from "@lucide/svelte/icons/search";
	import Keyboard from "@lucide/svelte/icons/keyboard";
	import CirclePlay from "@lucide/svelte/icons/circle-play";
	import Wrench from "@lucide/svelte/icons/wrench";
	import Flag from "@lucide/svelte/icons/flag";
	import ChevronRight from "@lucide/svelte/icons/chevron-right";
	import FixList from "$lib/components/FixList.svelte";
	import { api } from "$lib/api";
	import { data } from "$lib/data.svelte";
	import { go, nav, startTour } from "$lib/nav.svelte";

	const startHere = $derived(data.guides.filter((g) => g.category === "getting-started").slice(0, 3));

	function openReporting() {
		const report = data.guides.find((g) => g.tags.includes("breadbug") && g.category === "apps");
		if (report) go("guides", { guide: `${report.category}/${report.id}` });
		else api.reportProblem();
	}
</script>

<div class="page">
	<div class="page-col">
		<header>
			<h1 class="title">How can we help?</h1>
			<p class="lede">Guides, keybinds and fixes for BOS. Everything here works offline.</p>
		</header>

		<button class="searchbar" onclick={() => (nav.searchOpen = true)}>
			<Search size={18} /><span>Search guides, keybinds, commands and fixes</span><kbd>Ctrl K</kbd>
		</button>


		<div class="tiles">
			<button class="tile" onclick={() => go("keybinds")}>
				<Keyboard size={20} /><b>Keybinds</b><small>Every shortcut, read live from your config</small>
			</button>
			<button class="tile" onclick={startTour}>
				<CirclePlay size={20} /><b>Take the tour</b><small>Hands-on, on your real desktop</small>
			</button>
			<button class="tile" onclick={() => go("fix")}>
				<Wrench size={20} /><b>Fix a problem</b><small>Sound, Wi-Fi, updates and more</small>
			</button>
			<button class="tile" onclick={openReporting}>
				<Flag size={20} /><b>Report a problem</b><small>breadbug, and exactly what it sends</small>
			</button>
		</div>
		{#if nav.tourStatus}<p class="tour-status" role="status" aria-live="polite">{nav.tourStatus}</p>{/if}

		<section class="group" aria-labelledby="qf">
			<h2 class="group-title" id="qf">Quick fixes</h2>
			<FixList fixes={data.fixes} />
		</section>

		{#if startHere.length}
			<section class="group" aria-labelledby="sh">
				<h2 class="group-title" id="sh">Start here</h2>
				{#each startHere as g (g.id)}
					<button class="entry link" onclick={() => go("guides", { guide: `${g.category}/${g.id}` })}>
						<span class="lb"><b>{g.title}</b></span><small>{g.categoryTitle}</small><ChevronRight size={16} />
					</button>
				{/each}
			</section>
		{/if}
	</div>
</div>

<style>
	.searchbar { display: flex; align-items: center; gap: 10px; padding: 12px 14px; border-radius: var(--radius); background: var(--surface); border: 1px solid var(--card-line); color: var(--muted); text-align: left; cursor: pointer; }
	.searchbar span { flex: 1; }
	.searchbar kbd { font-family: ui-monospace, monospace; font-size: 10px; border: 1px solid var(--line); background: var(--bg-2); border-radius: 5px; padding: 2px 5px; }
	.searchbar:hover { color: var(--fg); }
	.tiles { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 10px; }
	.tour-status { margin: 0; padding: 8px 12px; border-radius: var(--radius-sm); background: var(--accent-soft); border: 1px solid var(--accent-edge); color: var(--fg); font-size: 12.5px; }
	.tile { display: flex; flex-direction: column; align-items: flex-start; gap: 6px; padding: 16px; border-radius: var(--radius); background: var(--surface); border: 1px solid var(--card-line); color: var(--fg); text-align: left; cursor: pointer; min-height: 104px; }
	.tile:hover { background: color-mix(in srgb, var(--surface) 88%, var(--fg)); }
	.tile :global(svg) { color: var(--accent); margin-bottom: 6px; }
	.tile b { font-weight: 400; }
	.tile small { color: var(--muted); font-size: 12px; line-height: 1.35; }
	.link { width: 100%; background: none; border-left: 0; border-right: 0; border-bottom: 0; color: var(--fg); text-align: left; cursor: pointer; }
	.entry.link:first-of-type { border-top: 0; }
	.link small { color: var(--muted); font-size: 12px; }
	.link :global(svg) { color: var(--muted); flex: none; }
	.link:hover .lb b { color: var(--accent); }
</style>
