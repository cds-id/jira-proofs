<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import PreviewPopup from './lib/PreviewPopup.svelte';
  import CardPicker from './lib/CardPicker.svelte';
  import Settings from './lib/Settings.svelte';

  const appWindow = getCurrentWindow();

  let showPreview = false;
  let showCardPicker = false;
  let showSettings = false;
  let captureResult: { file_path: string; filename: string; is_image: boolean } | null = null;

  async function showWindow() {
    await appWindow.show();
    await appWindow.setFocus();
  }

  async function hideWindow() {
    await appWindow.hide();
  }

  onMount(() => {
    listen<string>('tray-action', async (event) => {
      const action = event.payload;
      switch (action) {
        case 'screenshot_full':
          try {
            captureResult = await invoke('take_screenshot', { mode: 'full' });
            showPreview = true;
            await showWindow();
          } catch (e) {
            console.error('Screenshot failed:', e);
          }
          break;
        case 'screenshot_region':
          try {
            captureResult = await invoke('take_screenshot', { mode: 'region' });
            showPreview = true;
            await showWindow();
          } catch (e) {
            console.error('Region screenshot failed:', e);
          }
          break;
        case 'record_full':
          try {
            await invoke('start_recording', { mode: 'full' });
          } catch (e) {
            console.error('Recording failed:', e);
          }
          break;
        case 'record_region':
          try {
            await invoke('start_recording', { mode: 'region' });
          } catch (e) {
            console.error('Recording failed:', e);
          }
          break;
        case 'stop_recording':
          try {
            captureResult = await invoke('stop_recording');
            showPreview = true;
            await showWindow();
          } catch (e) {
            console.error('Stop recording failed:', e);
          }
          break;
        case 'set_card':
          showCardPicker = true;
          await showWindow();
          break;
        case 'settings':
          showSettings = true;
          await showWindow();
          break;
      }
    });
  });

  async function handleClose() {
    showPreview = false;
    captureResult = null;
    await hideWindow();
  }

  async function handleCardPickerClose() {
    showCardPicker = false;
    await hideWindow();
  }

  async function handleSettingsClose() {
    showSettings = false;
    await hideWindow();
  }
</script>

{#if showPreview && captureResult}
  <PreviewPopup
    filePath={captureResult.file_path}
    filename={captureResult.filename}
    isImage={captureResult.is_image}
    on:close={handleClose}
  />
{/if}
{#if showCardPicker}
  <CardPicker on:close={handleCardPickerClose} />
{/if}
{#if showSettings}
  <Settings on:close={handleSettingsClose} />
{/if}
