<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { mount, unmount } from 'svelte';
  import { setupHost, loadExtension, unloadExtension, getRegistration } from '$lib/sdk/loader';
  import { createVantaAPI } from '$lib/sdk/api';
  import type { ToastOptions } from '$lib/sdk/types';
  import ExtList from './sdk/ExtList.svelte';
  import ExtForm from './sdk/ExtForm.svelte';
  import ExtGrid from './sdk/ExtGrid.svelte';
  import ExtDetail from './sdk/ExtDetail.svelte';
  import ExtActionPanel from './sdk/ExtActionPanel.svelte';

  let {
    extId,
    commandName,
    extPath = '',
    onClose,
    onToast,
  }: {
    extId: string;
    commandName: string;
    extPath?: string;
    onClose?: () => void;
    onToast?: (options: ToastOptions) => void;
  } = $props();

  let container: HTMLDivElement | undefined = $state();
  let error: string | null = $state(null);
  let loading: boolean = $state(true);
  let mountedInstance: any = $state(null);
  let injectedStyleEl: HTMLStyleElement | null = $state(null);

  let navStack: Array<{ component: any; props?: Record<string, any> }> = $state([]);

  function handlePush(component: any, props?: Record<string, any>) {
    navStack.push({ component, props });
    mountTopOfStack();
  }

  function handlePop() {
    if (navStack.length > 1) {
      navStack.pop();
      mountTopOfStack();
    } else {
      onClose?.();
    }
  }

  function mountTopOfStack() {
    if (!container) return;
    cleanupMount();

    const top = navStack[navStack.length - 1];
    if (!top) return;

    try {
      mountedInstance = mount(top.component, {
        target: container,
        props: { ...(top.props ?? {}), api: currentApi },
      });
    } catch (err: any) {
      console.error(`[Vanta] Extension mount error:`, err);
      error = `Extension component crashed: ${err?.message ?? err}`;
    }
  }

  function cleanupMount() {
    if (mountedInstance) {
      try {
        unmount(mountedInstance);
      } catch { /* already unmounted */ }
      mountedInstance = null;
    }
    if (container) {
      container.innerHTML = '';
    }
  }

  let currentApi = $derived(createVantaAPI({
    extensionName: extId,
    extensionPath: extPath,
    onPush: handlePush,
    onPop: handlePop,
    onToast: (opts: ToastOptions) => onToast?.(opts),
  }));

  async function initExtension() {
    loading = true;
    error = null;

    try {
      setupHost({
        List: ExtList,
        Form: ExtForm,
        Grid: ExtGrid,
        Detail: ExtDetail,
        ActionPanel: ExtActionPanel,
      });

      let registration = getRegistration(extId) ?? undefined;
      if (!registration) {
        const bundleCode: string = await invoke('get_extension_bundle', { extId });
        registration = (await loadExtension(extId, bundleCode, { timeoutMs: 8000 })) ?? undefined;
      }

      if (!registration) {
        error = `Extension '${extId}' failed to load or register.`;
        loading = false;
        return;
      }

      const cmdReg = registration.commands[commandName];
      if (!cmdReg) {
        error = `Command '${commandName}' not found in extension '${extId}'.`;
        loading = false;
        return;
      }

      if (cmdReg.handler && !cmdReg.component) {
        await cmdReg.handler(currentApi);
        onClose?.();
        loading = false;
        return;
      }

      if (cmdReg.component) {
        const styles: string | null = await invoke('get_extension_styles', { extId });
        if (styles) {
          injectedStyleEl = document.createElement('style');
          injectedStyleEl.dataset.vantaExtStyle = extId;
          injectedStyleEl.textContent = styles;
          document.head.appendChild(injectedStyleEl);
        }

        navStack = [{ component: cmdReg.component, props: {} }];
        loading = false;

        await new Promise((r) => setTimeout(r, 0));
        mountTopOfStack();
        return;
      }

      error = `Command '${commandName}' has no handler or component.`;
    } catch (err: any) {
      console.error(`[Vanta] Extension init error:`, err);
      error = `Failed to initialize extension: ${err?.message ?? err}`;
    }

    loading = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      handlePop();
    }
  }

  $effect(() => {
    if (extId && commandName) {
      initExtension();
    }

    return () => {
      cleanupMount();
      if (injectedStyleEl?.parentNode) {
        injectedStyleEl.parentNode.removeChild(injectedStyleEl);
        injectedStyleEl = null;
      }
    };
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="extension-host">
  {#if loading}
    <div class="ext-loading">
      <div class="ext-spinner"></div>
      <span>Loading extension...</span>
    </div>
  {:else if error}
    <div class="ext-error">
      <i class="fa-solid fa-triangle-exclamation"></i>
      <p>{error}</p>
      <button onclick={() => onClose?.()}>Close</button>
    </div>
  {:else}
    <div class="ext-header">
      {#if navStack.length > 1}
        <button class="ext-back" onclick={handlePop} aria-label="Back">
          <i class="fa-solid fa-arrow-left"></i>
        </button>
      {/if}
      <span class="ext-title">{extId}</span>
      <button class="ext-close" onclick={() => onClose?.()} aria-label="Close">
        <i class="fa-solid fa-xmark"></i>
      </button>
    </div>
    <div class="ext-container" bind:this={container}></div>
  {/if}
</div>

<style>
  .extension-host {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: var(--vanta-bg, #000);
  }

  .ext-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    height: 100%;
    color: var(--vanta-text-dim, #888);
    font-size: 13px;
  }

  .ext-spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--vanta-border, rgba(255,255,255,0.08));
    border-top-color: var(--vanta-accent, #fff);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .ext-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    height: 100%;
    color: var(--vanta-text-dim, #888);
    font-size: 13px;
    padding: 24px;
    text-align: center;
  }

  .ext-error i {
    font-size: 28px;
    color: #f44;
  }

  .ext-error button {
    margin-top: 8px;
    padding: 6px 16px;
    background: var(--vanta-surface, #0a0a0a);
    border: 1px solid var(--vanta-border, rgba(255,255,255,0.08));
    border-radius: 6px;
    color: var(--vanta-text, #f5f5f5);
    cursor: pointer;
    font-size: 12px;
  }

  .ext-header {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    gap: 8px;
    border-bottom: 1px solid var(--vanta-border, rgba(255,255,255,0.08));
    flex-shrink: 0;
  }

  .ext-title {
    flex: 1;
    font-size: 12px;
    color: var(--vanta-text-dim, #888);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .ext-back,
  .ext-close {
    background: none;
    border: none;
    color: var(--vanta-text-dim, #888);
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
  }

  .ext-back:hover,
  .ext-close:hover {
    background: var(--vanta-surface, #0a0a0a);
    color: var(--vanta-text, #f5f5f5);
  }

  .ext-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }
</style>
