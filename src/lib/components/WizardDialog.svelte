<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { analyzeSample, type CandidateString, type SampleAnalysis } from "$lib/api";
  import { buildRule } from "$lib/wizard";
  import { humanSize, hexOffset } from "$lib/format";
  import { app } from "$lib/state.svelte";

  let analysis = $state<SampleAnalysis | null>(null);
  let selected = $state<Set<number>>(new Set());
  let loading = $state(false);
  let error = $state<string | null>(null);
  let conditionMode = $state<"all" | "any" | "n">("any");
  let nValue = $state(2);
  let categoryFilter = $state<string>("all");

  $effect(() => {
    if (app.wizardOpen && !analysis && !loading) {
      pickSample();
    }
  });

  async function pickSample() {
    const path = await open({ multiple: false, title: "Choose a sample to analyze" });
    if (!path) {
      if (!analysis) app.wizardOpen = false;
      return;
    }
    loading = true;
    error = null;
    try {
      analysis = await analyzeSample(path);
      // Pre-select the strongest IOC-like strings.
      selected = new Set(
        analysis.strings
          .map((s, i) => ({ s, i }))
          .filter(({ s }) => s.category !== "plain")
          .slice(0, 8)
          .map(({ i }) => i),
      );
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function close() {
    app.wizardOpen = false;
    analysis = null;
    selected = new Set();
    error = null;
  }

  function toggle(i: number) {
    const next = new Set(selected);
    if (next.has(i)) next.delete(i);
    else next.add(i);
    selected = next;
  }

  const categories = $derived.by(() => {
    if (!analysis) return [];
    const set = new Set(analysis.strings.map((s) => s.category));
    return ["all", ...set];
  });

  const visible = $derived.by(() => {
    if (!analysis) return [] as { s: CandidateString; i: number }[];
    return analysis.strings
      .map((s, i) => ({ s, i }))
      .filter(({ s }) => categoryFilter === "all" || s.category === categoryFilter);
  });

  const selectedList = $derived.by(() => {
    if (!analysis) return [] as CandidateString[];
    return [...selected].sort((a, b) => a - b).map((i) => analysis!.strings[i]);
  });

  function insertRule() {
    if (!analysis || selected.size === 0) return;
    const condition = conditionMode === "n" ? nValue : conditionMode;
    const rule = buildRule(analysis, selectedList, condition);
    app.setEditorContent(rule);
    app.currentRel = null;
    app.savedSource = null;
    app.showFlash("Rule generated");
    close();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={app.wizardOpen ? onKeydown : undefined} />

{#if app.wizardOpen}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <div class="dialog" aria-label="Rule wizard">
      <header class="head">
        <h2>Rule wizard</h2>
        {#if analysis}
          <button class="ghost small" onclick={pickSample}>Change sample</button>
        {/if}
      </header>

      {#if loading}
        <div class="center"><div class="spinner"></div><p>Analyzing…</p></div>
      {:else if error}
        <p class="error">{error}</p>
        <div class="buttons">
          <button class="ghost" onclick={close}>Close</button>
          <button class="primary" onclick={pickSample}>Pick another file</button>
        </div>
      {:else if analysis}
        <div class="meta">
          <span class="file">{analysis.fileName}</span>
          <span class="chip">{humanSize(analysis.size)}</span>
          {#if analysis.fileType}<span class="chip type">{analysis.fileType}</span>{/if}
          <span class="chip" title="Shannon entropy (0–8)">H {analysis.entropy.toFixed(2)}</span>
          <span class="chip mono" title="SHA-256">{analysis.sha256.slice(0, 12)}…</span>
        </div>

        <div class="filters">
          <span class="label">{selected.size} selected · filter:</span>
          {#each categories as cat}
            <button
              class="filter"
              class:active={categoryFilter === cat}
              onclick={() => (categoryFilter = cat)}>{cat}</button
            >
          {/each}
        </div>

        <div class="strings">
          {#each visible as { s, i } (i)}
            <label class="row" class:on={selected.has(i)}>
              <input type="checkbox" checked={selected.has(i)} onchange={() => toggle(i)} />
              <span class="cat cat-{s.category}">{s.category}</span>
              {#if s.kind === "wide"}<span class="wide">W</span>{/if}
              <span class="value" title={s.value}>{s.value}</span>
              <span class="off">{hexOffset(s.offset)}</span>
              {#if s.count > 1}<span class="count">×{s.count}</span>{/if}
            </label>
          {/each}
        </div>

        <div class="condition">
          <span class="label">Condition:</span>
          <label><input type="radio" bind:group={conditionMode} value="any" /> any of them</label>
          <label><input type="radio" bind:group={conditionMode} value="all" /> all of them</label>
          <label>
            <input type="radio" bind:group={conditionMode} value="n" />
            <input
              class="n"
              type="number"
              min="1"
              max={Math.max(1, selected.size)}
              bind:value={nValue}
              onfocus={() => (conditionMode = "n")}
            /> of them
          </label>
        </div>

        <div class="buttons">
          <span class="hint">Generates a rule into the editor — nothing is saved until you save it.</span>
          <span class="spacer"></span>
          <button class="ghost" onclick={close}>Cancel</button>
          <button class="primary" onclick={insertRule} disabled={selected.size === 0}>
            Generate rule
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: #0a0e14aa;
    z-index: 90;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dialog {
    width: 640px;
    max-height: 82vh;
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 12px 40px #00000066;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 40px;
    color: var(--muted);
  }
  .spinner {
    width: 24px;
    height: 24px;
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

  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .file {
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 600;
  }
  .chip {
    font-size: 11px;
    color: var(--muted);
    background: var(--bg1);
    border: 1px solid var(--border);
    padding: 1px 7px;
    border-radius: 10px;
  }
  .chip.type {
    color: var(--accent);
    border-color: #e8b33933;
  }
  .chip.mono {
    font-family: var(--font-mono);
  }

  .filters {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-wrap: wrap;
  }
  .label {
    font-size: 11.5px;
    color: var(--muted);
    margin-right: 4px;
  }
  .filter {
    background: none;
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--muted);
    font-size: 11px;
    padding: 1px 9px;
    cursor: pointer;
  }
  .filter.active {
    color: #1a1405;
    background: var(--accent);
    border-color: var(--accent);
  }

  .strings {
    flex: 1;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 7px;
    padding: 4px;
    min-height: 180px;
    max-height: 320px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 6px;
    border-radius: 5px;
    cursor: pointer;
    font-size: 12.5px;
  }
  .row:hover {
    background: var(--bg1);
  }
  .row.on {
    background: #1c2a4566;
  }

  .cat {
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0 5px;
    border-radius: 3px;
    flex-shrink: 0;
    width: 62px;
    text-align: center;
  }
  .cat-url,
  .cat-ip {
    color: #ff9a8b;
    background: #f8514918;
  }
  .cat-pdb,
  .cat-registry,
  .cat-path {
    color: #d2a8ff;
    background: #d2a8ff18;
  }
  .cat-useragent,
  .cat-email {
    color: #56d4dd;
    background: #56d4dd18;
  }
  .cat-plain {
    color: var(--muted);
    background: #7d8a9c18;
  }

  .wide {
    color: var(--accent);
    font-size: 9.5px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: #a5d6ff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .off {
    font-family: var(--font-mono);
    font-size: 11px;
    color: #4a5568;
    flex-shrink: 0;
  }
  .count {
    font-size: 10.5px;
    color: var(--muted);
    flex-shrink: 0;
  }

  .condition {
    display: flex;
    align-items: center;
    gap: 14px;
    flex-wrap: wrap;
    font-size: 12.5px;
  }
  .condition label {
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
  }
  .n {
    width: 46px;
    background: var(--bg0);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--text);
    font-family: var(--font-mono);
    padding: 2px 6px;
  }

  .buttons {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .hint {
    font-size: 11.5px;
    color: #4a5568;
  }
  .spacer {
    flex: 1;
  }

  .error {
    margin: 0;
    color: #ffb3ae;
    font-size: 12.5px;
  }

  .primary {
    background: var(--accent);
    color: #1a1405;
    border: none;
    border-radius: 6px;
    padding: 6px 16px;
    font-size: 12.5px;
    font-weight: 600;
    font-family: var(--font-ui);
    cursor: pointer;
  }
  .primary:hover:not(:disabled) {
    background: #f2c14e;
  }
  .primary:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .ghost,
  .ghost.small {
    background: none;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--muted);
    padding: 6px 12px;
    font-size: 12.5px;
    font-family: var(--font-ui);
    cursor: pointer;
  }
  .ghost.small {
    padding: 2px 10px;
    font-size: 11px;
  }
  .ghost:hover {
    color: var(--text);
  }
</style>
