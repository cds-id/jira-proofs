<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let onClose: () => void = () => {};

  interface JiraIssue {
    key: string;
    summary: string;
  }

  let query = '';
  let issues: JiraIssue[] = [];
  let loading = false;
  let errorMsg = '';
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  async function fetchIssues(q: string) {
    loading = true;
    errorMsg = '';
    try {
      issues = await invoke<JiraIssue[]>('search_jira_issues', { query: q });
    } catch (e) {
      errorMsg = String(e);
      issues = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchIssues('');
  });

  function handleInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      fetchIssues(query);
    }, 300);
  }

  async function selectCard(card: JiraIssue) {
    try {
      await invoke('set_active_card', { card });
      onClose();
    } catch (e) {
      errorMsg = String(e);
    }
  }

  async function clearCard() {
    try {
      await invoke('set_active_card', { card: null });
      onClose();
    } catch (e) {
      errorMsg = String(e);
    }
  }
</script>

<div class="popup-overlay">
  <div class="popup">
    <h2>Select Jira Card</h2>

    <div class="field">
      <input
        type="search"
        placeholder="Search issues…"
        bind:value={query}
        on:input={handleInput}
        autofocus
      />
    </div>

    {#if errorMsg}
      <div class="error-msg">{errorMsg}</div>
    {/if}

    {#if loading}
      <p style="color: var(--text); opacity: 0.6; font-size: 0.9rem;">Loading…</p>
    {:else if issues.length === 0}
      <p style="color: var(--text); opacity: 0.6; font-size: 0.9rem;">No issues found.</p>
    {:else}
      <div class="issue-list">
        {#each issues as issue}
          <button class="issue-item" on:click={() => selectCard(issue)}>
            <span class="issue-key">{issue.key}</span>
            <span class="issue-summary">{issue.summary}</span>
          </button>
        {/each}
      </div>
    {/if}

    <div class="button-row">
      <button class="btn-secondary" on:click={clearCard}>Clear Active Card</button>
      <button class="btn-cancel" on:click={onClose}>Cancel</button>
    </div>
  </div>
</div>
