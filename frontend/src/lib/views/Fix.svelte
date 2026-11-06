<script lang="ts">
	// Fix a problem: pick a symptom, answer a question or two, run the fix.
	// The trees are content (content/troubleshooting/_symptoms/*.toml). The
	// troubleshooter used to be a separate dialog; it is inline now.
	import Volume2 from "@lucide/svelte/icons/volume-2";
	import Wifi from "@lucide/svelte/icons/wifi";
	import CircleHelp from "@lucide/svelte/icons/circle-help";
	import Wrench from "@lucide/svelte/icons/wrench";
	import ChevronRight from "@lucide/svelte/icons/chevron-right";
	import FixList from "$lib/components/FixList.svelte";
	import { api } from "$lib/api";
	import { confirmFix } from "$lib/confirm.svelte";
	import { data } from "$lib/data.svelte";
	import { go, nav } from "$lib/nav.svelte";

	const OTHER = "__other";
	const ICONS: Record<string, typeof Wifi> = { "no-sound": Volume2, "wifi-wont-connect": Wifi };
	// Fixes the troubleshooters already offer don't need a second button here.
	const COVERED = new Set(["sound", "wifi"]);

	let node = $state("start");
	let result: "running" | "done" | "failed" | null = $state(null);

	const picked = $derived(nav.symptom || data.symptoms[0]?.id || OTHER);
	const tree = $derived(data.symptoms.find((t) => t.id === picked));
	const current = $derived(tree?.nodes.find((n) => n.id === node));
	const extraFixes = $derived(data.fixes.filter((f) => !COVERED.has(f.id)));
	const privacyGuide = $derived(data.guides.find((g) => g.tags.includes("privacy")));

	function pick(id: string) {
		nav.symptom = id;
	}
	// Any change of troubleshooter (a card here, or a result picked in
	// search while already on this page) starts its tree from the top.
	$effect(() => {
		void picked;
		reset();
	});
	function reset() {
		node = "start";
		result = null;
	}
	function goto(id: string) {
		node = id;
		result = null;
	}
	async function runFix() {
		if (!tree || !current || !current.fix) return;
		const requiresConfirm = current.requiresConfirm;
		// Declining leaves everything untouched: no command is invoked.
		if (!(await confirmFix({ label: current.fix, requiresConfirm }))) return;
		result = "running";
		try {
			result = (await api.runSymptomFix(tree.id, current.id, requiresConfirm)) ? "done" : "failed";
		} catch {
			result = "failed";
		}
	}
</script>

<div class="page">
	<div class="page-col">
		<header>
			<h1 class="title">Fix a problem</h1>
			<p class="lede">Pick what's wrong. Fixes run on this computer; nothing is sent anywhere.</p>
		</header>

		<div class="symptoms" role="group" aria-label="What's wrong">
			{#each data.symptoms as t (t.id)}
				{@const Icon = ICONS[t.id] ?? Wrench}
				<button class="sym" class:on={picked === t.id} aria-pressed={picked === t.id} onclick={() => pick(t.id)}>
					<span class="ic"><Icon size={18} /></span><b>{t.title}</b><small>{t.summary}</small>
				</button>
			{/each}
			<button class="sym" class:on={picked === OTHER} aria-pressed={picked === OTHER} onclick={() => pick(OTHER)}>
				<span class="ic"><CircleHelp size={18} /></span><b>Something else</b><small>An app crashed or froze</small>
			</button>
		</div>

		<section class="group walk" aria-live="polite">
			{#if picked === OTHER}
				<div class="prompt">Report it with breadbug. You see the whole report first, and nothing is sent unless you choose to.</div>
				<div class="fixrow">
					<span>Report a problem</span>
					<button class="btn primary" onclick={() => api.reportProblem()}>Open breadbug</button>
				</div>
			{:else if tree && current}
				<div class="crumb">
					<span>{tree.title}{node === "start" ? "" : " · next step"}</span>
					{#if node !== "start"}<button class="again" onclick={reset}>Start over</button>{/if}
				</div>
				<div class="prompt">{current.prompt}</div>
				{#each current.options as o (o.goto)}
					<button class="opt" onclick={() => goto(o.goto)}><span>{o.label}</span><ChevronRight size={16} /></button>
				{/each}
				{#if current.fix}
					<div class="fixrow">
						<span>{current.fix}</span>
						{#if result === "done"}<span class="st ok">Done. Did that fix it?</span>{/if}
						{#if result === "failed"}<span class="st bad">That didn't work. Try Start over or report it.</span>{/if}
						<button class="btn primary" disabled={result === "running"} onclick={runFix}>
							{result === "running" ? "Running..." : "Run fix"}
						</button>
					</div>
				{/if}
			{/if}
		</section>

		{#if extraFixes.length}
			<section class="group" aria-labelledby="ocf">
				<h2 class="group-title" id="ocf">One-click fixes</h2>
				<FixList fixes={extraFixes} />
			</section>
		{/if}

		<section class="group stuck">
			<div class="lb">
				<b>Still stuck? Report it.</b>
				<small>breadbug shows you the full report before anything is sent.
					{#if privacyGuide}<button class="linkish" onclick={() => go("guides", { guide: `${privacyGuide.category}/${privacyGuide.id}` })}>What a report contains</button>{/if}
				</small>
			</div>
			<button class="btn" onclick={() => api.reportProblem()}>Report a problem</button>
		</section>
	</div>
</div>

<style>
	.symptoms { display: grid; grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); gap: 10px; }
	.sym { display: flex; flex-direction: column; align-items: flex-start; gap: 6px; padding: 16px; border-radius: var(--radius); background: var(--surface); border: 1px solid var(--card-line); color: var(--fg); text-align: left; cursor: pointer; min-height: 104px; }
	.sym:hover { background: color-mix(in srgb, var(--surface) 88%, var(--fg)); }
	.sym.on { background: var(--accent-soft); border-color: var(--accent-edge); }
	.sym b { font-weight: 400; margin-top: 4px; }
	.sym small { color: var(--muted); font-size: 12px; }
	.ic { width: 34px; height: 34px; border-radius: 10px; background: var(--accent-soft); color: var(--accent); display: grid; place-items: center; }
	.walk { padding: 16px; display: flex; flex-direction: column; gap: 10px; }
	.crumb { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--muted); }
	.again, .linkish { background: none; border: 0; padding: 0; color: var(--accent); cursor: pointer; font-size: inherit; }
	.again { margin-left: auto; padding: 4px; }
	.prompt { font-family: var(--font-display, 'Outfit', sans-serif); font-size: 18px; font-weight: 500; line-height: 1.3; }
	.opt { display: flex; align-items: center; gap: 10px; width: 100%; min-height: 44px; padding: 10px 12px; border-radius: var(--radius-sm); border: 1px solid var(--line); background: var(--bg-2); color: var(--fg); text-align: left; cursor: pointer; }
	.opt span { flex: 1; }
	.opt:hover { border-color: var(--accent-edge); }
	.opt :global(svg) { color: var(--muted); }
	.fixrow { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; padding: 12px; border-radius: var(--radius-sm); background: var(--accent-soft); border: 1px solid var(--accent-edge); }
	.fixrow > span:first-child { flex: 1 1 200px; }
	.st { font-size: 12px; }
	.st.ok { color: var(--accent); }
	.st.bad { color: var(--red); }
	.stuck { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; padding: 16px; }
	.stuck .lb { flex: 1 1 300px; min-width: 0; }
	.stuck b { display: block; font-weight: 400; }
	.stuck small { display: block; color: var(--muted); font-size: 12px; margin-top: 2px; line-height: 1.5; }
</style>
