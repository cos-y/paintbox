<script lang="ts">
	import LazyPage from '$lib/components/LazyPage.svelte';
</script>

<LazyPage load={() => import('./SettingsPage.svelte')} />
