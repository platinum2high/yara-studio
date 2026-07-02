<script lang="ts">
  import { cancelScan } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import { formatTime } from "$lib/format";
  import FileCard from "./FileCard.svelte";

  let { onPickFiles }: { onPickFiles: () => void } = $props();

  function cancel() {
    if (app.scanId) cancelScan(app.scanId);
  }
</script>

<div class="panel">
  {#if app.scanning}
    <div class="center">
      <div class="spinner"></div>
      {#if app.scanProgress}
        <p class="progress-line">
          <b>{app.scanProgress.scanned}</b> scanned ·
          <b class:hot={app.scanProgress.matched > 0}>{app.scanProgress.matched}</b> matched
        </p>
        <p class="current-path" title={app.scanProgress.currentPath}>
          {app.scanProgress.currentPath}
        </p>
      {:else}
        <p>Scanning…</p>
      {/if}
      <button class="ghost cancel" onclick={cancel}>Cancel</button>
    </div>
  {:else if app.report}
    {@const r = app.report}
    <header class="summary">
      <span class="time">{formatTime(r.startedAtEpochMs)}</span>
      <span class="counts">
        {r.scannedFiles} file{r.scannedFiles === 1 ? "" : "s"} ·
        <b class:hot={r.matchedFiles > 0}>{r.matchedFiles} matched</b>
        {#if r.errorFiles > 0}
          · {r.errorFiles} error{r.errorFiles === 1 ? "" : "s"}
        {/if}
        · {r.ruleCount} rule{r.ruleCount === 1 ? "" : "s"} · {(r.durationMs / 1000).toFixed(
          r.durationMs < 10000 ? 1 : 0,
        )}s
      </span>
      <span class="spacer"></span>
      <button class="ghost" onclick={() => (app.report = null)}>Clear</button>
    </header>
    {#if app.scanError}
      <p class="scan-error">{app.scanError}</p>
    {/if}
    {#if r.cancelled}
      <p class="notice">Scan was cancelled — results below are partial.</p>
    {/if}
    {#if r.truncated}
      <p class="notice">Result list truncated to the first 5000 findings.</p>
    {/if}
    {#if r.results.length === 0}
      <div class="center">
        <p>
          {r.scannedFiles} file{r.scannedFiles === 1 ? "" : "s"} scanned — no matches, no
          errors.
        </p>
      </div>
    {/if}
    <div class="cards">
      {#each r.results as file (file.path)}
        <FileCard {file} />
      {/each}
    </div>
  {:else}
    <div class="center empty">
      {#if app.scanError}
        <p class="scan-error">{app.scanError}</p>
      {/if}
      <svg viewBox="0 0 64 64" class="drop-icon" aria-hidden="true">
        <rect x="8" y="8" width="48" height="48" rx="10" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="6 5" />
        <path d="M32 22v14m0 0-7-7m7 7 7-7" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M22 42h20" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" />
      </svg>
      <h2>Drop files to scan</h2>
      <p>They will be checked against the rules in the editor.<br />Nothing leaves this machine.</p>
      <button class="primary" onclick={onPickFiles}>Choose files…</button>
    </div>
  {/if}
</div>

<style>
  .panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg0);
    overflow: hidden;
  }

  .center {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--muted);
    padding: 24px;
    text-align: center;
  }

  .empty h2 {
    margin: 4px 0 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
  }
  .empty p {
    margin: 0;
    font-size: 13px;
    line-height: 1.5;
  }

  .drop-icon {
    width: 64px;
    height: 64px;
    color: #2c3648;
  }

  .progress-line {
    margin: 0;
    font-size: 13px;
  }
  .progress-line b {
    color: var(--text);
  }
  .progress-line b.hot {
    color: var(--red);
  }

  .current-path {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 11px;
    color: #4a5568;
    max-width: 90%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .cancel {
    margin-top: 10px;
  }

  .notice {
    margin: 0;
    padding: 8px 14px;
    color: var(--accent);
    background: #e8b33910;
    border-bottom: 1px solid #e8b33930;
    font-size: 12px;
  }

  .spinner {
    width: 26px;
    height: 26px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .summary {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
    font-size: 12.5px;
    color: var(--muted);
    flex-shrink: 0;
  }
  .time {
    font-family: var(--font-mono);
  }
  .counts b {
    color: var(--text);
    font-weight: 600;
  }
  .counts b.hot {
    color: var(--red);
  }
  .spacer {
    flex: 1;
  }

  .scan-error {
    margin: 0;
    padding: 10px 14px;
    color: #ffb3ae;
    background: #f8514915;
    border-bottom: 1px solid #f851493a;
    font-size: 12.5px;
  }

  .cards {
    flex: 1;
    overflow-y: auto;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  button.primary {
    margin-top: 8px;
    background: var(--accent);
    color: #1a1405;
    border: none;
    border-radius: 6px;
    padding: 7px 16px;
    font-size: 13px;
    font-weight: 600;
    font-family: var(--font-ui);
    cursor: pointer;
  }
  button.primary:hover {
    background: #f2c14e;
  }

  button.ghost {
    background: none;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--muted);
    padding: 3px 10px;
    font-size: 12px;
    font-family: var(--font-ui);
    cursor: pointer;
  }
  button.ghost:hover {
    color: var(--text);
    border-color: #2c3648;
  }
</style>
