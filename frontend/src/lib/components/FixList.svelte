<script lang="ts">
	// One-click fixes as rows with a Run button and the outcome beside it.
	import { api, type Fix } from "$lib/api";

	let { fixes }: { fixes: Fix[] } = $props();
	let status: Record<string, "running" | "done" | "failed"> = $state({});

	async function run(fix: Fix) {
		status[fix.id] = "running";
		try {
			status[fix.id] = (await api.runFix(fix.id)) ? "done" : "failed";
		} catch {
			status[fix.id] = "failed";
		}
	}

	const TEXT = { running: "Running...", done: "Done", failed: "Didn't work" };
</script>

{#each fixes as fix (fix.id)}
	<div class="entry">
		<div class="lb"><b>{fix.label}</b><small>{fix.description}</small></div>
		<span class="st {status[fix.id] ?? ''}" aria-live="polite">{status[fix.id] ? TEXT[status[fix.id]] : ""}</span>
		<button class="btn" disabled={status[fix.id] === "running"} onclick={() => run(fix)}>Run</button>
	</div>
{/each}

<style>
	.st { font-size: 12px; color: var(--muted); }
	.st.done { color: var(--accent); }
	.st.failed { color: var(--red); }
</style>
