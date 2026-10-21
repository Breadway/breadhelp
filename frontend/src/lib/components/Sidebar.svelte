<script lang="ts">
	// Same shell as bos-settings' sidebar (search button, items, uppercase
	// section labels, footer), so the two windows read as one desktop.
	import House from "@lucide/svelte/icons/house";
	import Keyboard from "@lucide/svelte/icons/keyboard";
	import BookOpen from "@lucide/svelte/icons/book-open";
	import Wrench from "@lucide/svelte/icons/wrench";
	import CirclePlay from "@lucide/svelte/icons/circle-play";
	import Flag from "@lucide/svelte/icons/flag";
	import Search from "@lucide/svelte/icons/search";
	import Type from "@lucide/svelte/icons/type";
	import { api, type Mode, type Page } from "$lib/api";
	import { data } from "$lib/data.svelte";
	import { go, nav } from "$lib/nav.svelte";

	const items: { page: Page; label: string; icon: typeof House; hint?: string }[] = [
		{ page: "home", label: "Home", icon: House },
		{ page: "keybinds", label: "Keybinds", icon: Keyboard, hint: "Super /" },
		{ page: "guides", label: "Guides", icon: BookOpen },
		{ page: "fix", label: "Fix a problem", icon: Wrench },
	];

	const modes: { value: Mode; label: string }[] = [
		{ value: "normal", label: "Normal" },
		{ value: "beginner", label: "Beginner" },
		{ value: "dad", label: "Larger text" },
		{ value: "compact", label: "Compact" },
	];

	function setMode(mode: Mode) {
		nav.mode = mode;
		api.setMode(mode);
	}
</script>

<nav class="sidebar" aria-label="Help sections">
	<button class="search" onclick={() => (nav.searchOpen = true)} aria-label="Search help">
		<Search size={17} /><span>Search help</span><kbd>Ctrl K</kbd>
	</button>
	<div class="nav">
		{#each items as item (item.page)}
			<button class="item" class:selected={nav.page === item.page} aria-current={nav.page === item.page ? "page" : undefined} onclick={() => go(item.page)}>
				<item.icon size={17} /><span class="label">{item.label}</span>{#if item.hint}<small class="hint">{item.hint}</small>{/if}
			</button>
		{/each}
		<div class="section-header">Getting started</div>
		<button class="item" onclick={() => api.startTour()}>
			<CirclePlay size={17} /><span class="label">Take the tour</span>
		</button>
		{#if data.setupNeeded}
			<button class="item" class:selected={nav.page === "setup"} aria-current={nav.page === "setup" ? "page" : undefined} onclick={() => go("setup")}>
				<Flag size={17} /><span class="label">Finish setup</span><small class="badge">{data.setupStepsLeft} left</small>
			</button>
		{/if}
	</div>
	<div class="foot">
		<Type size={17} aria-hidden="true" />
		<label for="mode">Display</label>
		<select id="mode" value={nav.mode} onchange={(e) => setMode((e.currentTarget as HTMLSelectElement).value as Mode)}>
			{#each modes as m (m.value)}<option value={m.value}>{m.label}</option>{/each}
		</select>
	</div>
</nav>

<style>
	.sidebar { width: 236px; flex: none; display: flex; flex-direction: column; min-height: 0; background: var(--bg-2); border-right: 1px solid var(--line); }
	.search { margin: 14px 12px 8px; display: flex; align-items: center; gap: 9px; padding: 9px 11px; border-radius: 10px; background: var(--surface-2); color: var(--muted); border: 1px solid var(--line); text-align: left; cursor: pointer; font-size: 13px; }
	.search span { flex: 1; }
	.search kbd, .hint { white-space: nowrap; font-size: 10px; color: var(--muted); }
	.search kbd { border: 1px solid var(--line); background: var(--surface); border-radius: 5px; padding: 2px 5px; font-family: ui-monospace, monospace; }
	.nav { flex: 1; overflow-y: auto; padding: 2px 10px 12px; scrollbar-width: thin; }
	.section-header { font-size: 10px; font-weight: 500; letter-spacing: 0.12em; text-transform: uppercase; color: var(--faint); padding: 14px 10px 6px; }
	.item { display: flex; align-items: center; gap: 11px; width: 100%; min-height: var(--target); padding: 7px 10px; border: 0; border-radius: 9px; color: var(--fg); background: transparent; text-align: left; font-size: 13px; cursor: pointer; }
	.item:hover { background: color-mix(in srgb, var(--fg) 4%, transparent); }
	.item.selected { background: var(--accent-soft); }
	.item.selected :global(svg) { color: var(--accent); }
	.item :global(svg) { flex: none; }
	.label { flex: 1; }
	.badge { font-size: 10px; padding: 1px 6px; border-radius: 99px; background: var(--accent); color: var(--on-accent); }
	.foot { display: flex; align-items: center; gap: 10px; padding: 12px 14px; border-top: 1px solid var(--line); font-size: 12.5px; color: var(--muted); }
	.foot select { margin-left: auto; background: var(--surface-2); color: var(--fg); border: 1px solid var(--line); border-radius: 8px; padding: 5px 8px; font-size: 12.5px; }
	@media (max-width: 760px) {
		.sidebar { width: 64px; }
		.item { justify-content: center; }
		.label, .hint, .badge, .section-header, .search span, .search kbd, .foot label { display: none; }
		.search { justify-content: center; margin: 12px 8px 8px; }
		/* The display picker stays usable as an invisible select over its icon. */
		.foot { justify-content: center; position: relative; }
		.foot select { position: absolute; inset: 0; margin: 0; opacity: 0; cursor: pointer; }
	}
</style>
