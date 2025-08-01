<script lang="ts">
	import {
		InputTokenAmount,
		WalletConnect,
		type VaultActionArgs
	} from '@rainlanguage/ui-components';
	import { Modal, Button } from 'flowbite-svelte';
	import { appKitModal, connected } from '$lib/stores/wagmi';
	import { validateAmount } from '$lib/services/validateAmount';
	import { fade } from 'svelte/transition';
	import truncateEthAddress from 'truncate-eth-address';
	import type { AccountBalance } from '@rainlanguage/orderbook';

	/**
	 * Modal component for depositing tokens into a vault.
	 * This component should only be used for deposit actions.
	 */
	export let open: boolean;
	export let args: VaultActionArgs;
	export let onSubmit: (amount: bigint) => void;

	const { vault, account } = args;

	let amount: bigint = 0n;
	let userBalance: AccountBalance = {
		balance: BigInt(0),
		formattedBalance: '0'
	} as unknown as AccountBalance;
	let errorMessage = '';
	let isCheckingCalldata = false;

	const getUserBalance = async () => {
		const balance = await vault.getOwnerBalance();
		if (balance.error) {
			errorMessage = balance.error.readableMsg;
			return;
		}
		userBalance = balance.value;
		return userBalance;
	};

	async function handleContinue() {
		onSubmit(amount);
		handleClose();
	}

	function handleClose() {
		open = false;
		amount = 0n;
	}

	$: validation = validateAmount(amount, userBalance.balance);
</script>

<Modal bind:open autoclose={false} size="md">
	<div class="space-y-4">
		<h3 class="text-xl font-medium" data-testid="modal-title">Deposit</h3>

		<div class="h-10">
			{#if account}
				{#await getUserBalance() then userBalance}
					{#if userBalance}
						<div in:fade class="w-full flex-col justify-between">
							<div class="flex justify-between">
								<p>
									Balance of connected wallet <span class="text-green-500"
										>{truncateEthAddress(account)}</span
									>
								</p>
								<p in:fade>
									{userBalance.formattedBalance}
									{vault.token.symbol}
								</p>
							</div>
							<div class="flex justify-between">
								<p>Balance of vault</p>
								<p in:fade>
									{`${vault.formattedBalance} ${vault.token.symbol}`}
								</p>
							</div>
						</div>
					{/if}
				{/await}
			{:else}
				<p>Connect your wallet to continue.</p>
			{/if}
		</div>
		<InputTokenAmount
			bind:value={amount}
			symbol={vault.token.symbol}
			decimals={Number(vault.token.decimals)}
			maxValue={userBalance.balance}
		/>
		<div class="flex flex-col justify-end gap-2">
			<div class="flex gap-2">
				<Button color="alternative" on:click={handleClose}>Cancel</Button>
				{#if account}
					<div class="flex flex-col gap-2">
						<Button
							color="blue"
							data-testid="deposit-button"
							on:click={handleContinue}
							disabled={!validation.isValid || isCheckingCalldata}
						>
							{#if isCheckingCalldata}
								Checking...
							{:else}
								Deposit
							{/if}
						</Button>
					</div>
				{:else}
					<WalletConnect {appKitModal} {connected} />
				{/if}
			</div>
			{#if errorMessage}
				<p data-testid="error-message">{errorMessage}</p>
			{/if}
			{#if validation.exceedsBalance}
				<p class="text-red-500" data-testid="amount-error">
					{validation.errorMessage}
				</p>
			{/if}
		</div>
	</div>
</Modal>
