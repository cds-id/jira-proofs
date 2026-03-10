<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { convertFileSrc } from '@tauri-apps/api/core';

  export let filePath: string;
  export let filename: string;
  export let isImage: boolean;

  const dispatch = createEventDispatcher();

  interface JiraIssue {
    key: string;
    summary: string;
  }

  interface Preset {
    name: string;
    template: string;
  }

  let presets: Preset[] = [];
  let activeCard: JiraIssue | null = null;
  let selectedPreset = '';
  let cardOverride = '';
  let description = '';
  let videoFormat: 'mp4' | 'gif' = 'mp4';
  let uploading = false;
  let errorMsg = '';

  const mediaSrc = convertFileSrc(filePath);

  onMount(async () => {
    try {
      const raw: [string, string][] = await invoke('get_presets');
      presets = raw.map(([name, template]) => ({ name, template }));
      if (presets.length > 0) {
        selectedPreset = presets[0].name;
      }
    } catch (e) {
      console.error('Failed to load presets:', e);
    }

    try {
      activeCard = await invoke<JiraIssue | null>('get_active_card');
      if (activeCard) {
        cardOverride = activeCard.key;
      }
    } catch (e) {
      console.error('Failed to load active card:', e);
    }
  });

  async function handleUpload() {
    const issueKey = cardOverride.trim();
    if (!issueKey) {
      errorMsg = 'Please enter or select a Jira issue key.';
      return;
    }
    if (!selectedPreset) {
      errorMsg = 'Please select a preset.';
      return;
    }

    uploading = true;
    errorMsg = '';

    try {
      let actualFilePath = filePath;
      let actualIsImage = isImage;

      // Convert to GIF if requested for video
      if (!isImage && videoFormat === 'gif') {
        const gifResult: { file_path: string; filename: string; is_image: boolean } =
          await invoke('convert_to_gif', { inputPath: filePath });
        actualFilePath = gifResult.file_path;
        actualIsImage = true;
      }

      await invoke('upload_and_post', {
        filePath: actualFilePath,
        issueKey,
        presetTitle: selectedPreset,
        description,
        isImage: actualIsImage,
      });

      dispatch('close');
    } catch (e) {
      errorMsg = String(e);
    } finally {
      uploading = false;
    }
  }

  async function handleSaveLocal() {
    dispatch('close');
  }

  function handleCancel() {
    dispatch('close');
  }
</script>

<div class="popup-overlay">
  <div class="popup">
    <h2>Preview — {filename}</h2>

    {#if isImage}
      <img class="preview" src={mediaSrc} alt={filename} />
    {:else}
      <!-- svelte-ignore a11y-media-has-caption -->
      <video class="preview" src={mediaSrc} controls></video>
    {/if}

    <div class="controls">
      {#if !isImage}
        <div class="field">
          <label>Format</label>
          <select bind:value={videoFormat}>
            <option value="mp4">MP4</option>
            <option value="gif">GIF (convert)</option>
          </select>
        </div>
      {/if}

      <div class="field">
        <label>Preset</label>
        <select bind:value={selectedPreset}>
          {#each presets as preset}
            <option value={preset.name}>{preset.name}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label>
          Jira Card
          {#if activeCard}
            <span style="color: var(--primary); margin-left: 0.4rem;">
              (active: {activeCard.key})
            </span>
          {/if}
        </label>
        <input
          type="text"
          placeholder="e.g. PROJ-123"
          bind:value={cardOverride}
        />
      </div>

      <div class="field">
        <label>Description (optional)</label>
        <textarea placeholder="Add a note..." bind:value={description}></textarea>
      </div>

      {#if errorMsg}
        <div class="error-msg">{errorMsg}</div>
      {/if}

      <div class="button-row">
        <button class="btn-cancel" on:click={handleCancel} disabled={uploading}>Cancel</button>
        <button class="btn-secondary" on:click={handleSaveLocal} disabled={uploading}>
          Save Local
        </button>
        <button class="btn-primary" on:click={handleUpload} disabled={uploading}>
          {uploading ? 'Uploading…' : 'Upload & Post'}
        </button>
      </div>
    </div>
  </div>
</div>
