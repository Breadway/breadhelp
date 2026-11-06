<script lang="ts">
	// First-run setup: its own full-window layout, not a help page, shown on
	// the first login's autostart (and from "Finish setup" until done). It
	// never takes over Super + /. Finishing or skipping hands off to the tour.
	import Flag from "@lucide/svelte/icons/flag";
	import Check from "@lucide/svelte/icons/check";
	import Keys from "$lib/components/Keys.svelte";
	import GuideBlocks from "$lib/components/GuideBlocks.svelte";
	import { api, type Guide, type Setup } from "$lib/api";
	import { go, nav, prefersSimple } from "$lib/nav.svelte";

	const STEPS = [
		{ title: "Welcome", heading: "Welcome to BOS" },
		{ title: "Time zone", heading: "Time zone" },
		{ title: "Network", heading: "Network" },
		{ title: "Updates", heading: "Updates" },
		{ title: "Snapshot", heading: "Snapshot" },
		{ title: "All set", heading: "All set" },
	];

	let setup: Setup | null = $state(null);
	let step = $state(0);
	let welcome: Guide | null = $state(null);
	let tzFilter = $state("");
	let tzPick = $state("");
	let tzStatus = $state("");
	let snapStatus = $state("");
	/// Shown while the tour process is starting, and kept on screen with the
	/// reason when it doesn't, instead of closing the window into silence.
	let finishStatus = $state("");
	let finishing = $state(false);

	$effect(() => {
		api.setup().then((s) => {
			setup = s;
			step = s.step;
			tzPick = s.timezone;
		});
		api.guide("getting-started", "01-what-is-bos", prefersSimple(nav.mode)).then((g) => (welcome = g));
	});

	const zones = $derived.by(() => {
		const q = tzFilter.trim().toLowerCase().replace(/\s+/g, "_");
		const all = setup?.timezones ?? [];
		return (q ? all.filter((z) => z.toLowerCase().includes(q)) : all).slice(0, 200);
	});

	/** Scroll the picked zone into view when the list first appears. */
	function showPicked(el: HTMLElement) {
		requestAnimationFrame(() => el.querySelector<HTMLElement>(".zone.on")?.scrollIntoView({ block: "center" }));
	}

	function goStep(i: number) {
		step = Math.max(0, Math.min(STEPS.length - 1, i));
		api.setSetupStep(step);
	}

	async function applyTz() {
		if (!tzPick) return;
		tzStatus = "Applying...";
		const r = await api.applyTimezone(tzPick);
		tzStatus = r.message;
		if (r.ok && setup) setup.timezone = tzPick;
	}

	async function snapshot() {
		snapStatus = "Creating a snapshot...";
		snapStatus = (await api.createSnapshot()).message;
	}

	/**
	 * Finish (or skip) setup and hand over to the tour. The status is set
	 * before awaiting so the user sees something happen immediately; if the
	 * tour process does not come up, the window stays put with the reason
	 * instead of closing on a desktop where nothing appears.
	 */
	async function finish() {
		if (finishing) return;
		finishing = true;
		finishStatus = "Starting the tour...";
		try {
			const r = await api.finishSetup();
			if (!r.ok) finishStatus = r.message;
		} catch (e) {
			finishStatus = `Couldn't start the tour: ${String(e)}`;
		} finally {
			finishing = false;
		}
	}
</script>

<div class="setup">
	<nav class="rail" aria-label="Setup steps">
		<div class="brand">
			<span class="mark"><Flag size={16} /></span>
			<div><b>Set up BOS</b><small>About two minutes</small></div>
		</div>
		{#each STEPS as s, i (s.title)}
			<button class="step" class:on={i === step} aria-current={i === step ? "step" : undefined} onclick={() => goStep(i)}>
				<span class="num" class:done={i < step} class:cur={i === step}>{#if i < step}<Check size={12} />{:else}{i + 1}{/if}</span>
				<span>{s.title}</span>
			</button>
		{/each}
		<div class="rail-foot">Keybinds any time: <Keys caps={["Super", "/"]} /></div>
	</nav>

	<main class="body">
		<header class="head">
			<div class="count">Step {step + 1} of {STEPS.length}</div>
			<h1 class="title">{STEPS[step].heading}</h1>
		</header>

		<div class="scroll">
			<div class="content">
				{#if step === 0}
					{#if welcome}
						<div class="welcome"><GuideBlocks blocks={welcome.blocks.slice(0, 4)} /></div>
					{:else}
						<p>BOS (the Bread Operating System) is a complete Hyprland desktop with the bread apps preinstalled. This short setup gets the clock, network and updates in place.</p>
					{/if}
				{:else if step === 1}
					{#if !setup || setup.timezones.length === 0}
						<p>Couldn't list time zones (is timedatectl available?). You can set this later in BOS Settings.</p>
					{:else}
						<p>Pick where you are so the clock and calendar are right. Applying may ask for your password.</p>
						<div class="current">Current: <b>{setup.timezone || "unknown"}</b></div>
						<label for="tzq" class="sr-only">Search time zones</label>
						<input id="tzq" class="field" type="search" bind:value={tzFilter} placeholder="Search for a city" />
						<div class="zones" role="listbox" aria-label="Time zones" use:showPicked>
							{#each zones as z (z)}
								<button class="zone" class:on={z === tzPick} role="option" aria-selected={z === tzPick} onclick={() => (tzPick = z)}>{z.replace(/_/g, " ")}</button>
							{/each}
						</div>
						<div class="row">
							<button class="btn primary" disabled={!tzPick || tzPick === setup.timezone} onclick={applyTz}>Apply {tzPick ? tzPick.replace(/_/g, " ") : "time zone"}</button>
							<span class="status" aria-live="polite">{tzStatus}</span>
						</div>
					{/if}
				{:else if step === 2}
					<p>Get online so BOS can check for updates. This window doesn't manage networks itself.</p>
					{#if setup?.networkTool}
						<div class="card-row"><span>Wi-Fi and network settings</span><button class="btn" onclick={() => api.openNetworkTool()}>{setup.networkTool.label}</button></div>
					{:else}
						<p class="dim">No network settings app found. Run <code>nmcli</code> in a terminal to connect.</p>
					{/if}
					{#if setup?.hasBreadcrumbs}<p class="dim">Location profiles (home, work, away) are handled by breadcrumbs once a network is saved.</p>{/if}
				{:else if step === 3}
					<p>When you're online, bring the system and the bread apps up to date. Nothing installs from this step.</p>
					{#if setup?.updatesTool}
						<div class="card-row"><span>Check for updates</span><button class="btn" onclick={() => api.openUpdatesTool()}>{setup.updatesTool.label}</button></div>
					{/if}
					<p class="dim">Or run <code>bos-update</code> in a terminal.</p>
				{:else if step === 4}
					{#if setup?.snapper}
						<p>A snapshot is a restore point. A first one named day-zero gives you a clean point to roll back to.</p>
						<div class="card-row"><span aria-live="polite">{snapStatus || "Create a snapshot of this fresh install"}</span><button class="btn" onclick={snapshot}>Create snapshot</button></div>
					{:else}
						<p>snapper isn't available here, so there's no first snapshot to make. You can skip this step.</p>
					{/if}
				{:else}
					<p>Next, a short hands-on tour of your real desktop: the bar, the launcher and the keys that matter. You can replay it from Help any time.</p>
				{/if}
			</div>
		</div>

		{#if finishStatus}
			<div class="notice" role="status" aria-live="polite">{finishStatus}</div>
		{/if}

		<footer class="foot">
			<button class="btn ghost" disabled={finishing} onclick={finish}>Skip setup</button>
			<button class="btn ghost" onclick={() => go("home")}>Later</button>
			<span class="sp"></span>
			{#if step > 0}<button class="btn" onclick={() => goStep(step - 1)}>Back</button>{/if}
			{#if step < STEPS.length - 1}
				<button class="btn primary" onclick={() => goStep(step + 1)}>Next</button>
			{:else}
				<button class="btn primary" disabled={finishing} onclick={finish}>{finishing ? "Starting the tour..." : "Start the tour"}</button>
			{/if}
		</footer>
	</main>
</div>

<style>
	.setup { display: flex; height: 100%; }
	.rail { flex: 0 0 236px; display: flex; flex-direction: column; gap: 2px; padding: 22px 12px; background: var(--bg-2); border-right: 1px solid var(--line); }
	.brand { display: flex; align-items: center; gap: 10px; padding: 0 10px 18px; }
	.brand b { display: block; font-family: var(--font-display, 'Outfit', sans-serif); font-weight: 500; font-size: 15px; }
	.brand small { color: var(--muted); font-size: 11.5px; }
	.mark { width: 30px; height: 30px; border-radius: 9px; background: var(--accent-soft); color: var(--accent); display: grid; place-items: center; }
	.step { display: flex; align-items: center; gap: 11px; width: 100%; min-height: var(--target); padding: 7px 10px; border: 0; border-radius: 9px; background: transparent; color: var(--fg); text-align: left; cursor: pointer; font-size: 13px; }
	.step:hover { background: color-mix(in srgb, var(--fg) 4%, transparent); }
	.step.on { background: var(--accent-soft); }
	.num { width: 20px; height: 20px; border-radius: 99px; display: grid; place-items: center; font-size: 11px; flex: none; border: 1px solid color-mix(in srgb, var(--fg) 22%, transparent); color: var(--muted); }
	.num.done, .num.cur { background: var(--accent); color: var(--on-accent); border-color: transparent; }
	.rail-foot { margin-top: auto; padding: 12px 10px 0; font-size: 12px; color: var(--muted); display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
	.body { flex: 1; min-width: 0; min-height: 0; display: flex; flex-direction: column; overflow: hidden; }
	.head { flex: none; padding: 28px 40px 4px; }
	.count { font-size: 12px; color: var(--muted); }
	.title { margin: 6px 0 0; }
	/* The only scrolling region. The footer lives outside it, so a long list
	   scrolls inside the body instead of pushing Next below the window edge. */
	.scroll { flex: 1 1 auto; min-height: 0; overflow-y: auto; padding: 8px 40px 4px; scrollbar-width: thin; scrollbar-color: var(--surface-2) transparent; }
	.content { max-width: 560px; display: flex; flex-direction: column; gap: 12px; padding-bottom: 16px; }
	.content p { margin: 0; line-height: 1.55; color: color-mix(in srgb, var(--fg) 80%, transparent); }
	.welcome :global(h2) { margin-top: 6px; }
	.dim { color: var(--muted) !important; font-size: 0.93em; }
	code { font-family: ui-monospace, monospace; background: var(--surface-2); border-radius: 5px; padding: 1px 5px; }
	.current { padding: 10px 12px; border-radius: var(--radius-sm); background: var(--accent-soft); border: 1px solid var(--accent-edge); }
	.current b { font-weight: 500; }
	.field { background: var(--surface); border: 1px solid var(--card-line); border-radius: var(--radius-sm); padding: 10px 12px; color: var(--fg); }
	.field:focus { border-color: var(--accent); }
	.zones { background: var(--surface); border: 1px solid var(--card-line); border-radius: var(--radius); max-height: 200px; overflow-y: auto; }
	.zone { display: block; width: 100%; min-height: 36px; padding: 8px 12px; border: 0; background: transparent; color: var(--fg); text-align: left; cursor: pointer; }
	.zone + .zone { border-top: 1px solid var(--line); }
	.zone:hover { background: color-mix(in srgb, var(--fg) 4%, transparent); }
	.zone.on { background: var(--accent-soft); }
	.row, .card-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
	.card-row { padding: 14px 16px; border-radius: var(--radius); background: var(--surface); border: 1px solid var(--card-line); }
	.card-row span { flex: 1 1 200px; }
	.status { font-size: 12.5px; color: var(--muted); }
	.notice { flex: none; margin: 0 40px 8px; padding: 10px 12px; border-radius: var(--radius-sm); background: var(--accent-soft); border: 1px solid var(--accent-edge); color: var(--fg); font-size: 12.5px; line-height: 1.4; }
	.foot { flex: none; padding: 14px 40px 20px; border-top: 1px solid var(--line); background: var(--bg); display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
	.sp { flex: 1; }
	@media (max-width: 760px) {
		.setup { flex-direction: column; }
		.rail { flex: none; flex-direction: row; overflow-x: auto; padding: 8px; }
		.brand, .rail-foot { display: none; }
		.step { width: auto; flex: none; }
		.head { padding: 14px 16px 2px; }
		.scroll { padding: 8px 16px 4px; }
		.notice { margin: 0 16px 8px; }
		.foot { padding: 12px 16px 16px; }
	}
</style>
