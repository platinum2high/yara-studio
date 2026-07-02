<script lang="ts">
  import { readHexRegion } from "$lib/api";
  import { hexOffset } from "$lib/format";

  let {
    path,
    matchOffset,
    matchLength,
  }: { path: string; matchOffset: number; matchLength: number } = $props();

  const ROW = 16;
  const CONTEXT = 128;
  const STEP = 256;
  const MAX_REGION = 4096;

  let start = $state(0);
  let bytes = $state<number[]>([]);
  let fileSize = $state(0);
  let loadError = $state<string | null>(null);
  let loading = $state(false);

  function parseHex(hex: string): number[] {
    const out = new Array<number>(hex.length / 2);
    for (let i = 0; i < out.length; i++) {
      out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
    }
    return out;
  }

  async function load(from: number, length: number) {
    loading = true;
    loadError = null;
    try {
      const region = await readHexRegion(path, from, length);
      start = region.start;
      bytes = parseHex(region.bytesHex);
      fileSize = region.fileSize;
    } catch (e) {
      loadError = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    const from = Math.max(0, Math.floor((matchOffset - CONTEXT) / ROW) * ROW);
    const length = Math.min(matchLength + CONTEXT * 2, MAX_REGION);
    load(from, length);
  });

  const end = $derived(start + bytes.length);
  const canExtend = $derived(bytes.length < MAX_REGION);

  function extendUp() {
    const newStart = Math.max(0, start - STEP);
    load(newStart, Math.min(end - newStart, MAX_REGION));
  }

  function extendDown() {
    load(start, Math.min(end + STEP - start, MAX_REGION));
  }

  const rows = $derived.by(() => {
    const out: { offset: number; cells: { value: number; hit: boolean }[] }[] = [];
    for (let i = 0; i < bytes.length; i += ROW) {
      out.push({
        offset: start + i,
        cells: bytes.slice(i, i + ROW).map((value, j) => ({
          value,
          hit:
            start + i + j >= matchOffset &&
            start + i + j < matchOffset + matchLength,
        })),
      });
    }
    return out;
  });

  function printable(value: number): string {
    return value >= 0x20 && value < 0x7f ? String.fromCharCode(value) : "·";
  }
</script>

<div class="hexview">
  {#if loadError}
    <p class="error">{loadError}</p>
  {:else}
    {#if start > 0 && canExtend}
      <button class="more" onclick={extendUp} disabled={loading}>
        ↑ {hexOffset(Math.max(0, start - STEP))} … show earlier bytes
      </button>
    {/if}
    <div class="grid" role="table">
      {#each rows as row (row.offset)}
        <div class="row" role="row">
          <span class="off">{hexOffset(row.offset)}</span>
          <span class="hx">
            {#each row.cells as cell, i}
              <span class="cell" class:hit={cell.hit} class:gap={i === 8}
                >{cell.value.toString(16).toUpperCase().padStart(2, "0")}</span
              >
            {/each}
          </span>
          <span class="asc">
            {#each row.cells as cell}
              <span class:hit={cell.hit}>{printable(cell.value)}</span>
            {/each}
          </span>
        </div>
      {/each}
    </div>
    {#if end < fileSize && canExtend}
      <button class="more" onclick={extendDown} disabled={loading}>
        ↓ show later bytes … {hexOffset(Math.min(fileSize, end + STEP))}
      </button>
    {/if}
  {/if}
</div>

<style>
  .hexview {
    background: var(--bg0);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 10px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    line-height: 1.6;
    overflow-x: auto;
  }

  .row {
    display: flex;
    gap: 14px;
    white-space: nowrap;
  }

  .off {
    color: #4a5568;
    user-select: none;
  }

  .hx {
    display: inline-flex;
  }

  .cell {
    color: #8b98a9;
    padding: 0 3px;
    border-radius: 2px;
  }
  .cell.gap {
    margin-left: 8px;
  }
  .cell.hit {
    color: #1a1405;
    background: var(--accent);
  }

  .asc {
    color: #a5d6ff;
    letter-spacing: 0.05em;
  }
  .asc .hit {
    color: var(--accent);
    font-weight: 700;
  }

  .more {
    display: block;
    width: 100%;
    background: none;
    border: none;
    color: var(--muted);
    font-family: var(--font-mono);
    font-size: 11px;
    cursor: pointer;
    padding: 3px 0;
    text-align: left;
  }
  .more:hover {
    color: var(--accent);
  }

  .error {
    color: #ffb3ae;
    font-size: 12px;
    margin: 0;
  }
</style>
