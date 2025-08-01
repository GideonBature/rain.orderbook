<script lang="ts" generics="T">
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';
	import type { QueryObserverResult } from '@tanstack/svelte-query';
	import type { Readable } from 'svelte/store';
	import DropdownActiveNetworks from './dropdown/DropdownActiveNetworks.svelte';
	import { page } from '$app/stores';
	import { isEmpty } from 'lodash';
	import { Alert } from 'flowbite-svelte';
	import type { Address, RaindexVaultToken } from '@rainlanguage/orderbook';
	import Tooltip from './Tooltip.svelte';
	import CheckboxActiveOrders from './checkbox/CheckboxActiveOrders.svelte';
	import DropdownOrderListAccounts from './dropdown/DropdownOrderListAccounts.svelte';
	import DropdownTokensFilter from './dropdown/DropdownTokensFilter.svelte';
	import InputOrderHash from './input/InputOrderHash.svelte';
	import CheckboxZeroBalanceVault from './CheckboxZeroBalanceVault.svelte';
	import CheckboxMyItemsOnly from '$lib/components/CheckboxMyItemsOnly.svelte';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import type { AppStoresInterface } from '$lib/types/appStores';

	export let hideZeroBalanceVaults: AppStoresInterface['hideZeroBalanceVaults'];
	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'];
	export let showMyItemsOnly: AppStoresInterface['showMyItemsOnly'];
	export let selectedChainIds: AppStoresInterface['selectedChainIds'];
	export let showInactiveOrders: AppStoresInterface['showInactiveOrders'];
	export let orderHash: AppStoresInterface['orderHash'];
	export let activeTokens: AppStoresInterface['activeTokens'];
	export let selectedTokens: Address[];
	export let tokensQuery: Readable<QueryObserverResult<RaindexVaultToken[], Error>>;

	$: isVaultsPage = $page.url.pathname === '/vaults';
	$: isOrdersPage = $page.url.pathname === '/orders';

	const { account } = useAccount();
	const raindexClient = useRaindexClient();

	$: networks = raindexClient.getAllNetworks();
	$: accounts = raindexClient.getAllAccounts();
</script>

<div
	class="grid w-full items-center gap-4 md:flex md:justify-end lg:min-w-[600px]"
	style="grid-template-columns: repeat(2, minmax(0, 1fr));"
>
	{#if networks.error || isEmpty(networks.value)}
		<Alert color="gray" data-testid="no-networks-alert" class="w-full">
			No networks added to <a class="underline" href="/settings">settings</a>
		</Alert>
	{:else}
		{#if !accounts.error && accounts.value.size === 0}
			<div class="mt-4 w-full lg:w-auto" data-testid="my-items-only">
				<CheckboxMyItemsOnly context={isVaultsPage ? 'vaults' : 'orders'} {showMyItemsOnly} />
				{#if !$account}
					<Tooltip>Connect a wallet to filter by {isVaultsPage ? 'vault' : 'order'} owner</Tooltip>
				{/if}
			</div>
		{/if}
		{#if isVaultsPage}
			<div class="mt-4 w-full lg:w-auto">
				<CheckboxZeroBalanceVault {hideZeroBalanceVaults} />
			</div>
		{/if}

		{#if isOrdersPage}
			<InputOrderHash {orderHash} />
			<div class="mt-4">
				<CheckboxActiveOrders {showInactiveOrders} />
			</div>
		{/if}
		{#if !accounts.error && accounts.value.size > 0}
			<DropdownOrderListAccounts {activeAccountsItems} />
		{/if}
		<DropdownTokensFilter {tokensQuery} {activeTokens} {selectedTokens} label="Tokens" />
		<DropdownActiveNetworks {selectedChainIds} />
	{/if}
</div>
