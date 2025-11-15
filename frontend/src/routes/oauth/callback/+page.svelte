<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	// Parser les query params
	let status = $state<'success' | 'error' | 'loading'>('loading');
	let provider = $state<string>('');
	let errorMessage = $state<string>('');

	onMount(() => {
		// Récupérer les paramètres de l'URL
		const urlParams = new URLSearchParams(window.location.search);
		const urlStatus = urlParams.get('status');
		const urlProvider = urlParams.get('provider');
		const urlError = urlParams.get('error') || urlParams.get('message');

		if (urlStatus === 'success' && urlProvider) {
			status = 'success';
			provider = urlProvider;
			sendMessageToParent('success', urlProvider);
		} else if (urlStatus === 'error') {
			status = 'error';
			errorMessage = urlError || 'Une erreur est survenue lors de la connexion OAuth';
			sendMessageToParent('error', urlProvider || 'unknown', errorMessage);
		} else {
			status = 'error';
			errorMessage = 'Paramètres de callback invalides';
			sendMessageToParent('error', 'unknown', errorMessage);
		}

		// Auto-fermeture après 2 secondes
		const timer = setTimeout(() => {
			window.close();
		}, 2000);

		return () => clearTimeout(timer);
	});

	function sendMessageToParent(
		messageStatus: 'success' | 'error',
		messageProvider: string,
		error?: string
	) {
		// Vérifier que nous sommes bien dans une popup
		if (!window.opener) {
			console.warn('OAuth callback: window.opener is null, cannot send message to parent');
			return;
		}

		// Construire le message
		const message = {
			type: 'oauth-callback',
			status: messageStatus,
			provider: messageProvider,
			...(error && { error })
		};

		// Envoyer le message au parent (même origine uniquement)
		try {
			window.opener.postMessage(message, window.location.origin);
		} catch (err) {
			console.error('Failed to send message to parent window:', err);
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-gray-50 to-gray-100">
	<div class="bg-white rounded-lg shadow-lg p-8 max-w-md w-full mx-4">
		{#if status === 'loading'}
			<div class="text-center">
				<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
				<h2 class="text-xl font-semibold text-gray-800">Traitement en cours...</h2>
			</div>
		{:else if status === 'success'}
			<div class="text-center">
				<div class="bg-green-100 rounded-full p-3 w-16 h-16 mx-auto mb-4 flex items-center justify-center">
					<svg class="w-10 h-10 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
					</svg>
				</div>
				<h2 class="text-2xl font-bold text-gray-800 mb-2">Connexion réussie !</h2>
				<p class="text-gray-600 mb-4">
					Votre compte <span class="font-semibold capitalize">{provider}</span> a été connecté avec succès.
				</p>
				<p class="text-sm text-gray-500">Cette fenêtre va se fermer automatiquement...</p>
			</div>
		{:else}
			<div class="text-center">
				<div class="bg-red-100 rounded-full p-3 w-16 h-16 mx-auto mb-4 flex items-center justify-center">
					<svg class="w-10 h-10 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
					</svg>
				</div>
				<h2 class="text-2xl font-bold text-gray-800 mb-2">Échec de la connexion</h2>
				<p class="text-gray-600 mb-4">{errorMessage}</p>
				<p class="text-sm text-gray-500">Vous pouvez fermer cette fenêtre.</p>
			</div>
		{/if}
	</div>
</div>
