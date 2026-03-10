<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let onClose: () => void = () => {};

  let hotkeys: { action: string; key: string }[] = [];

  onMount(async () => {
    try {
      const result = await invoke<[string, string][]>('get_hotkeys');
      hotkeys = result.map(([action, key]) => ({ action, key }));
    } catch (e) {
      console.error('Failed to load hotkeys:', e);
    }
  });
</script>

<div class="popup-overlay">
  <div class="popup">
    <h2>Settings</h2>

    <section>
      <h3>Global Hotkeys</h3>
      <p class="hint">
        Hotkeys are read-only. To change them, edit
        <code>~/.config/jira-proofs/config.toml</code>.
      </p>
      <ul class="hotkey-list">
        {#each hotkeys as hk}
          <li class="hotkey-row">
            <span class="hotkey-action">{hk.action}</span>
            <kbd>{hk.key}</kbd>
          </li>
        {/each}
      </ul>
    </section>

    <div class="button-row">
      <button class="btn-primary" on:click={onClose}>Close</button>
    </div>
  </div>
</div>

<style>
  h3 {
    font-size: 0.95rem;
    color: var(--text);
    margin-bottom: 0.4rem;
  }

  .hint {
    font-size: 0.8rem;
    color: var(--text);
    opacity: 0.6;
    margin-bottom: 0.75rem;
    line-height: 1.4;
  }

  .hint code {
    font-family: monospace;
    background: var(--bg);
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    border: 1px solid var(--border);
  }

  .hotkey-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .hotkey-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.4rem 0;
    border-bottom: 1px solid var(--border);
  }

  .hotkey-row:last-child {
    border-bottom: none;
  }

  .hotkey-action {
    font-size: 0.9rem;
  }

  kbd {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.2rem 0.5rem;
    font-size: 0.8rem;
    font-family: monospace;
    white-space: nowrap;
    color: var(--primary);
  }
</style>
