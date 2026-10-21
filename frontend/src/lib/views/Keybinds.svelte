<script lang="ts">
	// The keybind reference (Super + / lands here). Rows come straight from
	// ~/.config/hypr/binds.json, with related binds grouped by the backend.
	import Search from "@lucide/svelte/icons/search";
	import Keys from "$lib/components/Keys.svelte";
	import { api, type KeyRow } from "$lib/api";
	import { data } from "$lib/data.svelte";

	let filter = $state("");
	let category = $state("all");
	let open: string | null = $state(null);
	let rebind = $state("");
	let message = $state("");

	const kb = $derived(data.keybinds);
	const visible = $derived.by(() => {
		const q = filter.trim().toLowerCase();
		return (kb?.rows ?? []).filter(
			(r) =>
				(category === "all" || r.category === category) &&
				(!q || `${r.label} ${r.caps.join(" ")} ${r.command ?? ""}`.toLowerCase().includes(q)),
		);
	});
	const groups = $derived(
		(kb?.categories ?? [])
			.map((c) => ({ ...c, rows: visible.filter((r) => r.category === c.id) }))
			.filter((g) => g.rows.length > 0),
	);

	function key(r: KeyRow) {
		return r.comboId ?? `${r.category}:${r.label}`;
	}

	function toggle(r: KeyRow) {
		open = open === key(r) ? null : key(r);
		rebind = "";
		message = "";
	}

	async function apply(r: KeyRow) {
		if (!r.comboId) return;
		try {
			await api.rebindKeybind(r.comboId, rebind);
			message = `Changed until you log out or Hyprland reloads.`;
		} catch (e) {
			message = String(e);
		}
	}
</script>

<div class="page">
	<div class="page-col wide">
		<header class="head">
			<div>
				<h1 class="title">Keybinds</h1>
				<p class="lede">Read live from your Hyprland config, so this list always matches what is bound.</p>
			</div>
			{#if data.hasSettings}<button class="btn" onclick={() => api.openSettings("keybinds")}>Edit in Settings</button>{/if}
		</header>

		<div class="filter">
			<Search size={16} />
			<label for="kbq" class="sr-only">Filter keybinds</label>
			<input id="kbq" type="search" bind:value={filter} placeholder="Filter by key or action, e.g. screenshot" />
		</div>

		<div class="chips" role="group" aria-label="Category">
			<button class="chip" class:on={category === "all"} aria-pressed={category === "all"} onclick={() => (category = "all")}>All</button>
			{#each kb?.categories ?? [] as c (c.id)}
				<button class="chip" class:on={category === c.id} aria-pressed={category === c.id} onclick={() => (category = c.id)}>{c.title}</button>
			{/each}
		</div>

		{#if kb && kb.rows.length === 0}
			<div class="group empty">No keybinds found. breadhelp reads them from {kb.source}.</div>
		{:else if groups.length === 0}
			<div class="group empty">Nothing matches "{filter}".</div>
		{/if}

		<div class="grid">
			{#each groups as g (g.id)}
				<section class="group card" aria-labelledby={`kb-${g.id}`}>
					<h2 class="group-title" id={`kb-${g.id}`}>{g.title}</h2>
					{#each g.rows as r (key(r))}
						{@const isOpen = open === key(r)}
						{@const hasMore = !!(r.command || r.canTry || r.canRebind)}
						<div class="row-wrap">
							{#if hasMore}
								<button class="row" class:on={isOpen} aria-expanded={isOpen} onclick={() => toggle(r)}>
									<Keys caps={r.caps} />
									<span class="what">{r.label}</span>
								</button>
							{:else}
								<div class="row static">
									<Keys caps={r.caps} />
									<span class="what">{r.label}</span>
								</div>
							{/if}
							{#if isOpen}
								<div class="more">
									{#if r.command}<div class="cmd">Runs <code>{r.command}</code></div>{/if}
									<div class="acts">
										{#if r.canTry && r.comboId}<button class="btn primary" onclick={() => api.tryKeybind(r.comboId!)}>Try it</button>{/if}
										{#if r.canRebind}
											<label for="rb" class="rb-label">Change for this session</label>
											<input id="rb" class="rb" bind:value={rebind} placeholder="e.g. super + shift + u" onkeydown={(e) => e.key === "Enter" && apply(r)} />
											<button class="btn" disabled={!rebind.trim()} onclick={() => apply(r)}>Apply</button>
										{/if}
									</div>
									<div class="note">{message || "A change here resets on reload or login. Use Edit in Settings to keep it."}</div>
								</div>
							{/if}
						</div>
					{/each}
				</section>
			{/each}
		</div>

		{#if kb}<p class="source">Source: {kb.source} · {kb.bindCount} bindings, related ones grouped</p>{/if}
	</div>
</div>

<style>
	.head { display: flex; align-items: flex-end; gap: 16px; flex-wrap: wrap; }
	.head > div { flex: 1 1 320px; min-width: 0; }
	.filter { display: flex; align-items: center; gap: 10px; padding: 9px 12px; border-radius: var(--radius-sm); background: var(--surface); border: 1px solid var(--card-line); color: var(--muted); }
	.filter input { flex: 1; min-width: 0; background: none; border: 0; outline: none; color: var(--fg); font-size: 13px; }
	.chips { display: flex; gap: 6px; flex-wrap: wrap; }
	.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(330px, 1fr)); gap: 12px; align-items: start; }
	.card { padding: 12px 6px 6px; }
	.card .group-title { margin: 0 10px 4px; }
	.row { display: flex; align-items: center; gap: 12px; width: 100%; min-height: 42px; padding: 8px 10px; border: 0; border-radius: 9px; background: transparent; color: var(--fg); text-align: left; cursor: pointer; }
	.row.static { cursor: default; }
	button.row:hover { background: color-mix(in srgb, var(--fg) 4%, transparent); }
	.row.on, .row.on:hover { background: var(--accent-soft); }
	.row :global(.keys) { flex: none; min-width: 136px; }
	.what { flex: 1; min-width: 0; }
	.more { margin: 2px 10px 10px; padding: 12px; border-radius: var(--radius-sm); background: var(--bg-2); border: 1px solid var(--line); display: flex; flex-direction: column; gap: 10px; }
	.cmd { font-size: 12px; color: var(--muted); }
	.cmd code { font-family: ui-monospace, monospace; color: var(--fg); word-break: break-all; }
	.acts { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }
	.rb-label { font-size: 12px; color: var(--muted); }
	.rb { width: 150px; background: var(--bg); border: 1px solid var(--line); border-radius: 9px; padding: 7px 9px; color: var(--fg); font-size: 12.5px; }
	.rb:focus { border-color: var(--accent); }
	.note { font-size: 11.5px; color: var(--muted); }
	.empty { padding: 16px; color: var(--muted); }
	.source { margin: 0; font-size: 12px; color: var(--muted); }
</style>
