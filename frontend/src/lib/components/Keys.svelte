<script lang="ts">
	// A shortcut as keycaps, one per key. The label is for screen readers,
	// which would otherwise read "Super" "comma" as unrelated words.
	let { caps }: { caps: string[] } = $props();
</script>

<span class="keys" role="img" aria-label={caps.join(" plus ")}>
	{#each caps as cap, i (i)}<kbd class="kc" aria-hidden="true">{cap}</kbd>{/each}
</span>

<style>
	.keys { display: inline-flex; gap: 4px; flex-wrap: wrap; vertical-align: middle; }
</style>
