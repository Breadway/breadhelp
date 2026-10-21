<script lang="ts">
	import { onMount } from "svelte";
	import { listen } from "@tauri-apps/api/event";
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import Sidebar from "$lib/components/Sidebar.svelte";
	import SearchPalette from "$lib/components/SearchPalette.svelte";
	import Home from "$lib/views/Home.svelte";
	import Keybinds from "$lib/views/Keybinds.svelte";
	import Guides from "$lib/views/Guides.svelte";
	import Fix from "$lib/views/Fix.svelte";
	import Setup from "$lib/views/Setup.svelte";
	import { api, type Open, type Page } from "$lib/api";
	import { refresh } from "$lib/data.svelte";
	import { go, nav } from "$lib/nav.svelte";
	import { initTheme } from "$lib/theme";
	import "$lib/styles/app.css";

	const VIEWS = { home: Home, keybinds: Keybinds, guides: Guides, fix: Fix } as const;
	const TITLES: Record<Page, string> = { home: "BOS Help", keybinds: "Keybinds", guides: "Guides", fix: "Fix a problem", setup: "Set up BOS" };

	let content: HTMLElement | undefined = $state();
	const View = $derived(nav.page === "setup" ? null : VIEWS[nav.page]);

	function applyOpen(open: Open) {
		if (open.page) go(open.page);
		nav.suggestion = open.suggestion;
	}

	onMount(() => {
		initTheme();
		api.prefs().then((p) => (nav.mode = p.mode));
		// The page to land on is read with a command, not an event: an event
		// sent before this listener exists would be lost.
		api.openState().then(applyOpen);
		refresh();

		const unlisten = [
			// Every show: land on the requested page and refetch, so edits to
			// binds.json or guides since the last open are there.
			listen<Open>("open", (e) => {
				applyOpen(e.payload);
				refresh();
			}),
			listen<string>("suggestion", (e) => (nav.suggestion = e.payload)),
			listen<string>("screenshot-set-view", (e) => {
				if (e.payload === "search") nav.searchOpen = true;
				else go(e.payload as Page);
			}),
		];

		const onKey = (e: KeyboardEvent) => {
			if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
				e.preventDefault();
				nav.searchOpen = !nav.searchOpen;
			} else if (e.key === "Escape" && !e.defaultPrevented && !nav.searchOpen && nav.page !== "setup") {
				// Esc in a text field (keybind filter, rebind box) leaves the
				// field; anywhere else it dismisses the window, a cheatsheet you
				// close as fast as you summoned it.
				const t = e.target as HTMLElement | null;
				if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT")) {
					t.blur();
					return;
				}
				api.hideWindow();
			}
		};
		window.addEventListener("keydown", onKey);
		return () => {
			unlisten.forEach((p) => p.then((f) => f()));
			window.removeEventListener("keydown", onKey);
		};
	});

	$effect(() => {
		const root = document.documentElement;
		root.classList.remove("mode-beginner", "mode-dad", "mode-compact");
		if (nav.mode !== "normal") root.classList.add(`mode-${nav.mode}`);
	});

	$effect(() => {
		getCurrentWindow().setTitle(TITLES[nav.page]).catch(() => {});
	});

	// A new page starts at the top.
	$effect(() => {
		void nav.page;
		if (content) content.scrollTop = 0;
	});
</script>

{#if nav.page === "setup"}
	<Setup />
{:else}
	<div class="shell">
		<Sidebar />
		<main class="content" bind:this={content}>
			{#if View}<View />{/if}
		</main>
	</div>
{/if}

{#if nav.searchOpen}<SearchPalette />{/if}

<style>
	.shell { display: flex; height: 100%; }
	.content { flex: 1; min-width: 0; overflow-y: auto; scrollbar-width: thin; scrollbar-color: var(--surface-2) transparent; }
</style>
