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
          break;
        }
        case 'set_card':
          showCardPicker = true;
          showPreview = false;
          showSettings = false;
          break;
        case 'settings':
          showSettings = true;
          showPreview = false;
          showCardPicker = false;
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
    await appWindow.hide();
  }

  onMount(() => {
    pollInterval = setInterval(checkPending, 500);
    checkPending();
  });

  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval);
  });
</script>

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
