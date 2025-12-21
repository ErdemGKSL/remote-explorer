<script lang="ts">
    import { Toaster } from '$lib/components/ui/sonner';
	import './layout.css';

	const { children } = $props();
</script>

<Toaster />
{@render children()}
