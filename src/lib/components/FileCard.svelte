<script lang="ts">
  import type { FileResult } from "$lib/api";
  import { hexOffset, humanSize, shortHash } from "$lib/format";
  import FileStripe from "./FileStripe.svelte";
  import HexViewer from "./HexViewer.svelte";

  let { file }: { file: FileResult } = $props();

  let copied = $state(false);
  let expandedMatch = $state<string | null>(null);

  function toggleMatch(key: string) {
    expandedMatch = expandedMatch === key ? null : key;
  }

  async function copyHash() {
    if (!file.sha256) return;
    await navigator.clipboard.writeText(file.sha256);
    copied = true;
    setTimeout(() => (copied = false), 1200);
  }

  function metaText(meta: Record<string, unknown>, key: string): string | null {
    const value = meta[key];
    return typeof value === "string" && value.length > 0 ? value : null;
  }
</script>

<article class="card" data-status={file.status}>
  <header>
    <span class="dot"></span>
    <span class="name" title={file.path}>{file.fileName}</span>
    {#if file.status === "matched"}
      <span class="badge matched">
        {file.ruleMatches.length} rule{file.ruleMatches.length === 1 ? "" : "s"}
      </span>
    {:else if file.status === "clean"}
      <span class="badge clean">clean</span>
    {:else}
      <span class="badge error">error</span>
    {/if}
    <span class="spacer"></span>
    <span class="stat">{humanSize(file.size)}</span>
    <span class="stat">{file.durationMs} ms</span>
  </header>

  <FileStripe size={file.size} ruleMatches={file.ruleMatches} />

  {#if file.sha256}
    <button class="hash" onclick={copyHash} title="Copy SHA-256">
      <span class="hash-label">sha256</span>
      <code>{shortHash(file.sha256)}</code>
      <span class="copy-hint">{copied ? "copied" : "copy"}</span>
    </button>
  {/if}

  {#if file.error}
    <p class="file-error">{file.error}</p>
  {/if}

  {#each file.ruleMatches as rule (rule.identifier)}
    <section class="rule">
      <div class="rule-head">
        <span class="rule-name">{rule.identifier}</span>
        {#each rule.tags as tag}
          <span class="tag">{tag}</span>
        {/each}
      </div>
      {#if metaText(rule.meta, "description")}
        <p class="rule-desc">{metaText(rule.meta, "description")}</p>
      {/if}

      {#if rule.stringMatches.length > 0}
        <table>
          <thead>
            <tr>
              <th>string</th>
              <th>offset</th>
              <th>bytes</th>
              <th>ascii</th>
            </tr>
          </thead>
          <tbody>
            {#each rule.stringMatches as m}
              {@const key = `${rule.identifier}:${m.identifier}:${m.offset}`}
              <tr
                class="match-row"
                class:open={expandedMatch === key}
                onclick={() => toggleMatch(key)}
                title="Click to inspect bytes in file context"
              >
                <td class="ident">
                  <span class="chevron">{expandedMatch === key ? "▾" : "▸"}</span>
                  {m.identifier}
                  {#if m.xorKey !== null}
                    <span class="xor" title="Matched under XOR">
                      xor 0x{m.xorKey.toString(16).toUpperCase().padStart(2, "0")}
                    </span>
                  {/if}
                </td>
                <td class="offset">{hexOffset(m.offset)}</td>
                <td class="bytes">
                  <span class="ctx">{m.contextBeforeHex}</span>
                  <span class="hit">{m.matchedHex}</span>
                  <span class="ctx">{m.contextAfterHex}</span>
                  {#if m.truncated}<span class="trunc">+{m.length - 64} B</span>{/if}
                </td>
                <td class="ascii">{m.matchedAscii}</td>
              </tr>
              {#if expandedMatch === key}
                <tr class="hex-row">
                  <td colspan="4">
                    <HexViewer
                      path={file.path}
                      matchOffset={m.offset}
                      matchLength={m.length}
                    />
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      {:else}
        <p class="rule-desc">Matched by condition only — no string patterns fired.</p>
      {/if}
    </section>
  {/each}
</article>

<style>
  .card {
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  header {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .card[data-status="matched"] .dot {
    background: var(--red);
    box-shadow: 0 0 6px #f8514966;
  }
  .card[data-status="clean"] .dot {
    background: var(--green);
  }
  .card[data-status="error"] .dot {
    background: var(--muted);
  }

  .name {
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .badge {
    font-size: 11px;
    font-weight: 600;
    padding: 1px 8px;
    border-radius: 10px;
    flex-shrink: 0;
  }
  .badge.matched {
    color: #ffb3ae;
    background: #f8514922;
  }
  .badge.clean {
    color: #7ee787;
    background: #3fb95022;
  }
  .badge.error {
    color: var(--muted);
    background: #7d8a9c22;
  }

  .spacer {
    flex: 1;
  }

  .stat {
    color: var(--muted);
    font-size: 12px;
    flex-shrink: 0;
  }

  .hash {
    display: flex;
    align-items: center;
    gap: 8px;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    color: var(--muted);
    font-size: 12px;
    width: fit-content;
  }
  .hash-label {
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.08em;
  }
  .hash code {
    font-family: var(--font-mono);
    color: var(--text);
  }
  .copy-hint {
    opacity: 0;
    transition: opacity 0.15s;
    color: var(--accent);
  }
  .hash:hover .copy-hint {
    opacity: 1;
  }

  .file-error {
    color: #ffb3ae;
    font-size: 12.5px;
    margin: 0;
  }

  .rule {
    border-top: 1px solid var(--border);
    padding-top: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .rule-head {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .rule-name {
    font-family: var(--font-mono);
    font-weight: 700;
    font-size: 13px;
    color: #7ee787;
  }

  .tag {
    font-family: var(--font-mono);
    font-size: 11px;
    color: #56d4dd;
    background: #56d4dd18;
    padding: 0 6px;
    border-radius: 4px;
  }

  .rule-desc {
    color: var(--muted);
    font-size: 12px;
    margin: 0;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  th {
    text-align: left;
    color: var(--muted);
    font-weight: 500;
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 2px 10px 4px 0;
  }

  td {
    padding: 3px 10px 3px 0;
    vertical-align: top;
    border-top: 1px solid #1e263466;
  }

  .match-row {
    cursor: pointer;
  }
  .match-row:hover td {
    background: #12182399;
  }
  .match-row.open td {
    background: #121823;
  }

  .chevron {
    color: var(--muted);
    font-size: 10px;
    display: inline-block;
    width: 12px;
  }

  .hex-row td {
    padding: 6px 0 8px 12px;
  }

  .ident {
    color: var(--accent);
    white-space: nowrap;
  }

  .xor {
    color: #d2a8ff;
    background: #d2a8ff18;
    font-size: 10.5px;
    padding: 0 5px;
    border-radius: 4px;
    margin-left: 4px;
  }

  .offset {
    color: #79c0ff;
    white-space: nowrap;
  }

  .bytes {
    word-break: break-all;
    max-width: 420px;
  }
  .ctx {
    color: #4a5568;
  }
  .hit {
    color: #ffdf8e;
    background: #e8b33919;
    border-radius: 3px;
    padding: 0 2px;
  }
  .trunc {
    color: var(--muted);
    font-size: 10.5px;
    margin-left: 4px;
  }

  .ascii {
    color: #a5d6ff;
    word-break: break-all;
  }
</style>
