<script lang="ts">
	let title = 'Run Sous BPM';
</script>

<svelte:head>
	<title>{title}</title>
	<meta name="description" content="Music-fitness analytics platform" />
</svelte:head>

<section>
	<h1>Welcome to {title}</h1>
	<p>Correlate your music with your fitness performance</p>
</section>

<style>
	section {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex: 0.6;
	}

	h1 {
		width: 100%;
	}
</style>