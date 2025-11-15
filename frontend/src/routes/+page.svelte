<script lang="ts">
	import { Button } from '$lib/components/ui/button';

	const title = 'Run Sous BPM';
</script>

<svelte:head>
	<title>{title}</title>
	<meta name="description" content="Visualize your music on your running routes" />
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-background to-muted/20">
	<div class="flex flex-col items-center gap-8 px-4 max-w-2xl w-full">
		<div class="text-center space-y-4">
			<h1 class="text-7xl font-bold tracking-tight bg-gradient-to-br from-foreground to-foreground/60 bg-clip-text text-transparent">
				{title}
			</h1>
			<p class="text-xl text-muted-foreground">
				Visualize your music on your running routes
			</p>
		</div>

		<div class="flex gap-4">
			<Button href="/auth/login" size="lg" variant="default">
				Login
			</Button>
			<Button href="/auth/register" size="lg" variant="outline">
				Register
			</Button>
		</div>
	</div>
</div>