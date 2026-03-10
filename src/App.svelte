<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount, onDestroy } from 'svelte';
  import PreviewPopup from './lib/PreviewPopup.svelte';
  import CardPicker from './lib/CardPicker.svelte';
  import Settings from './lib/Settings.svelte';

  const appWindow = getCurrentWindow();

  let showPreview = false;
  let showCardPicker = false;
  let showSettings = false;
  let captureResult: { file_path: string; filename: string; is_image: boolean } | null = null;
  let title = 'Jira Proofs';

  interface PendingCapture {
    type: 'capture';
    file_path: string;
    filename: string;
    is_image: boolean;
  }
  interface PendingSetCard {
    type: 'set_card';
  }
  interface PendingSettings {
    type: 'settings';
  }
  type PendingAction = PendingCapture | PendingSetCard | PendingSettings;

  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function checkPending() {
    try {
      const pending: PendingAction | null = await invoke('get_pending_action');
      if (!pending) return;
      switch (pending.type) {
        case 'capture': {
          const cap = pending as PendingCapture;
          captureResult = {
            file_path: cap.file_path,
            filename: cap.filename,
            is_image: cap.is_image,
          };
          showPreview = true;
          showCardPicker = false;
          showSettings = false;
          title = 'Preview';
          break;
        }
        case 'set_card':
          showCardPicker = true;
          showPreview = false;
          showSettings = false;
          title = 'Select Card';
          break;
        case 'settings':
          showSettings = true;
          showPreview = false;
          showCardPicker = false;
          title = 'Settings';
          break;
      }
    } catch (e) {
      console.error('Failed to get pending action:', e);
    }
  }

  async function hideWindow() {
    showPreview = false;
    showCardPicker = false;
    showSettings = false;
    captureResult = null;
    title = 'Jira Proofs';
    await appWindow.hide();
  }

  async function minimizeWindow() {
    await appWindow.minimize();
  }

  onMount(() => {
    pollInterval = setInterval(checkPending, 500);
    checkPending();
  });

  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval);
  });
</script>

<div class="window">
  <div class="titlebar" data-tauri-drag-region>
    <span class="titlebar-title" data-tauri-drag-region>{title}</span>
    <div class="titlebar-buttons">
      <button class="titlebar-btn minimize" on:click={minimizeWindow}>&#x2013;</button>
      <button class="titlebar-btn close" on:click={hideWindow}>&#x2715;</button>
    </div>
  </div>

  <div class="content">
    {#if showPreview && captureResult}
      <PreviewPopup
        filePath={captureResult.file_path}
        filename={captureResult.filename}
        isImage={captureResult.is_image}
        onClose={hideWindow}
      />
    {:else if showCardPicker}
      <CardPicker onClose={hideWindow} />
    {:else if showSettings}
      <Settings onClose={hideWindow} />
    {/if}
  </div>
</div>

<style>
  .window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg);
  }

  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 0 0.5rem 0 0.75rem;
    background: var(--surface);
    user-select: none;
    flex-shrink: 0;
  }

  .titlebar-title {
    font-size: 0.8rem;
    color: var(--text);
    opacity: 0.8;
    pointer-events: none;
  }

  .titlebar-buttons {
    display: flex;
    gap: 2px;
  }

  .titlebar-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 0.85rem;
    cursor: pointer;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition: all 0.1s;
  }

  .titlebar-btn:hover {
    opacity: 1;
    background: var(--border);
  }

  .titlebar-btn.close:hover {
    background: var(--error);
    color: white;
  }

  .content {
    flex: 1;
    overflow-y: auto;
  }
</style>
