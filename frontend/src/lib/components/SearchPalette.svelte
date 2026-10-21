<script lang="ts">
	// Ctrl+K search over fixes, troubleshooters, guides, keybinds and the
	// command gallery, with a preview of the highlighted result. Replaces the
	// old "Ask" tab.
	import Search from "@lucide/svelte/icons/search";
	import Keys from "./Keys.svelte";
	import { api } from "$lib/api";
	import { searchEntries } from "$lib/data.svelte";
	import { go, nav } from "$lib/nav.svelte";
	import { flat, search, type Entry } from "$lib/search";

	let query = $state("");
	let selected = $state(0);
	let status = $state("");
	let input: HTMLInputElement | undefined = $state();

	const entries = $derived(searchEntries());
	const groups = $derived(search(entries, query));
	const items = $derived(flat(groups));
	const current = $derived(items[Math.min(selected, items.length - 1)] as Entry | undefined);

	$effect(() => {
		void query;
		selected = 0;
		status = "";
	});
	$effect(() => {
		input?.focus();
	});

	const KIND_LABEL: Record<Entry["kind"], string> = { fix: "Fix", symptom: "Troubleshooter", guide: "Guide", keybind: "Keybind", command: "Command" };
	const ACTION: Record<Entry["kind"], string> = { fix: "Run fix", symptom: "Start", guide: "Read guide", keybind: "Show in Keybinds", command: "Open" };

	async function activate(entry: Entry | undefined) {
		if (!entry) return;
		switch (entry.kind) {
			case "fix":
				status = `${entry.title}...`;
				status = (await api.runFix(entry.id)) ? `${entry.title}: done.` : `${entry.title}: something went wrong.`;
				break;
			case "symptom":
				go("fix", { symptom: entry.id });
				break;
			case "guide":
				go("guides", { guide: entry.id });
				break;
			case "keybind":
				go("keybinds");
				break;
			case "command":
				await api.runCommand(entry.id);
				nav.searchOpen = false;
				break;
		}
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === "Escape") {
			e.preventDefault();
			nav.searchOpen = false;
		} else if (e.key === "ArrowDown") {
			e.preventDefault();
			selected = Math.min(selected + 1, items.length - 1);
		} else if (e.key === "ArrowUp") {
			e.preventDefault();
			selected = Math.max(selected - 1, 0);
		} else if (e.key === "Enter") {
			e.preventDefault();
			activate(current);
		}
	}
</script>

<div class="backdrop" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) nav.searchOpen = false; }}>
	<div class="palette" role="dialog" aria-modal="true" aria-label="Search help" tabindex="-1" onkeydown={onKey}>
		<div class="bar">
			<Search size={18} />
			<label for="q" class="sr-only">Search help</label>
			<input id="q" bind:this={input} bind:value={query} type="search" autocomplete="off" spellcheck="false" placeholder="Search guides, keybinds, commands and fixes" aria-controls="results" />
			<kbd class="kc">Esc</kbd>
		</div>
		<div class="body">
			<div class="results" id="results" role="listbox" aria-label="Results">
				{#each groups as group (group.title)}
					<div class="group-label">{group.title}</div>
					{#each group.items as entry (entry.kind + entry.id)}
						{@const index = items.indexOf(entry)}
						<button class="res" class:on={index === selected} role="option" aria-selected={index === selected} onmouseenter={() => (selected = index)} onclick={() => activate(entry)}>
							<span class="dot {entry.kind}" aria-hidden="true"></span>
							<span class="t">{entry.title}</span>
							<small>{entry.hint}</small>
						</button>
					{/each}
				{:else}
					<p class="empty">Nothing matches. Try another word, or open <button class="link" onclick={() => go("fix")}>Fix a problem</button>.</p>
				{/each}
			</div>
			<div class="preview" aria-live="polite">
				{#if current}
					<div class="kind">{KIND_LABEL[current.kind]}</div>
					<h2>{current.title}</h2>
					{#if current.body}<p>{current.body}</p>{/if}
					{#if current.caps}<Keys caps={current.caps} />{/if}
					<div class="acts">
						<button class="btn primary" onclick={() => activate(current)}>{ACTION[current.kind]}</button>
						{#if status}<span class="status">{status}</span>{/if}
					</div>
					<div class="help"><kbd class="kc">↑</kbd> <kbd class="kc">↓</kbd> to move, <kbd class="kc">Enter</kbd> to open</div>
				{/if}
			</div>
		</div>
	</div>
</div>

<style>
	.backdrop { position: fixed; inset: 0; z-index: 20; background: rgba(0, 0, 0, 0.6); display: flex; justify-content: center; align-items: flex-start; padding: 56px 16px 16px; }
	.palette { width: 100%; max-width: 760px; background: color-mix(in srgb, var(--surface) 85%, var(--bg)); border: 1px solid var(--card-line); border-radius: var(--radius); box-shadow: 0 24px 60px rgba(0, 0, 0, 0.5); overflow: hidden; }
	.bar { display: flex; align-items: center; gap: 10px; padding: 12px 16px; border-bottom: 1px solid var(--line); color: var(--accent); }
	.bar input { flex: 1; min-width: 0; background: none; border: 0; outline: none; color: var(--fg); font-size: 15px; }
	.body { display: flex; flex-wrap: wrap; min-height: 400px; }
	.results { flex: 1 1 300px; min-width: 0; padding: 8px; border-right: 1px solid var(--line); max-height: min(460px, 70vh); overflow-y: auto; }
	.group-label { font-size: 10.5px; font-weight: 500; letter-spacing: 0.1em; text-transform: uppercase; color: var(--muted); padding: 10px 10px 4px; }
	.res { display: flex; align-items: center; gap: 10px; width: 100%; min-height: 40px; padding: 8px 10px; border: 0; border-radius: 9px; background: transparent; color: var(--fg); text-align: left; cursor: pointer; }
	.res.on { background: var(--accent-soft); }
	.res .t { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.res small { color: var(--muted); font-size: 11.5px; white-space: nowrap; }
	.dot { width: 8px; height: 8px; border-radius: 99px; flex: none; background: var(--muted); }
	.dot.fix, .dot.symptom { background: var(--accent); }
	.dot.guide { background: #8fb3d9; }
	.dot.keybind { background: #c9a6d8; }
	.dot.command { background: #9fc79a; }
	.preview { flex: 1 1 260px; min-width: 0; padding: 20px; display: flex; flex-direction: column; gap: 10px; }
	.kind { font-size: 10.5px; font-weight: 500; letter-spacing: 0.1em; text-transform: uppercase; color: var(--accent); }
	.preview h2 { font-family: var(--font-display, 'Outfit', sans-serif); font-size: 20px; }
	.preview p { margin: 0; color: color-mix(in srgb, var(--fg) 78%, transparent); line-height: 1.5; }
	.acts { margin-top: auto; display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
	.status { color: var(--muted); font-size: 12.5px; }
	.help { font-size: 11.5px; color: var(--muted); }
	.empty { padding: 20px 12px; color: var(--muted); }
	.link { background: none; border: 0; padding: 0; color: var(--accent); cursor: pointer; text-decoration: underline; }
</style>
