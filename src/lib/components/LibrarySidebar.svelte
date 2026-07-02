<script lang="ts">
  import { onMount } from "svelte";
  import type { LibraryEntry } from "$lib/api";
  import {
    createCollection,
    deleteCollection,
    deleteEntry,
    loadEntry,
    newRule,
    refreshLibrary,
  } from "$lib/library";
  import { app } from "$lib/state.svelte";

  let query = $state("");
  let newCollectionMode = $state(false);
  let newCollectionName = $state("");
  let collapsed = $state<Record<string, boolean>>({});

  onMount(() => {
    refreshLibrary();
  });

  function matches(entry: LibraryEntry): boolean {
    if (query.trim() === "") return true;
    const q = query.toLowerCase();
    return (
      entry.fileName.toLowerCase().includes(q) ||
      entry.ruleNames.some((r) => r.toLowerCase().includes(q)) ||
      entry.tags.some((t) => t.toLowerCase().includes(q)) ||
      (entry.description ?? "").toLowerCase().includes(q)
    );
  }

  async function submitCollection() {
    const name = newCollectionName.trim();
    if (name !== "") await createCollection(name);
    newCollectionMode = false;
    newCollectionName = "";
  }

  function displayName(entry: LibraryEntry): string {
    return entry.fileName.replace(/\.(yar|yara)$/, "");
  }
</script>

{#snippet entryRow(entry: LibraryEntry)}
  <div
    class="entry"
    class:active={app.currentRel === entry.rel}
    role="button"
    tabindex="0"
    onclick={() => loadEntry(entry.rel)}
    onkeydown={(e) => e.key === "Enter" && loadEntry(entry.rel)}
    title={entry.description ?? entry.ruleNames.join(", ")}
  >
    <span class="entry-name">
      {#if !entry.compiles}<span class="broken" title="Does not compile">✗</span>{/if}
      {displayName(entry)}
    </span>
    {#if entry.tags.length > 0}
      <span class="entry-tags">
        {#each entry.tags.slice(0, 3) as tag}
          <span class="tag">{tag}</span>
        {/each}
      </span>
    {/if}
    <button
      class="row-action"
      title="Delete rule"
      onclick={(e) => {
        e.stopPropagation();
        deleteEntry(entry.rel);
      }}>✕</button
    >
  </div>
{/snippet}

<aside>
  <header>
    <span class="title">Library</span>
    <span class="spacer"></span>
    <button class="icon" title="New rule" onclick={newRule}>＋</button>
    <button
      class="icon"
      title="New collection"
      onclick={() => (newCollectionMode = true)}>🗀</button
    >
  </header>

  <input class="search" placeholder="Filter by name, rule, tag…" bind:value={query} />

  {#if newCollectionMode}
    <form
      class="new-collection"
      onsubmit={(e) => {
        e.preventDefault();
        submitCollection();
      }}
    >
      <!-- svelte-ignore a11y_autofocus -->
      <input
        autofocus
        placeholder="collection name"
        bind:value={newCollectionName}
        onblur={submitCollection}
      />
    </form>
  {/if}

  <div class="tree">
    {#if app.libraryTree}
      {#each app.libraryTree.entries.filter(matches) as entry (entry.rel)}
        {@render entryRow(entry)}
      {/each}

      {#each app.libraryTree.collections as collection (collection.name)}
        {@const visible = collection.entries.filter(matches)}
        <div class="collection">
          <div
            class="collection-head"
            role="button"
            tabindex="0"
            onclick={() => (collapsed[collection.name] = !collapsed[collection.name])}
            onkeydown={(e) =>
              e.key === "Enter" &&
              (collapsed[collection.name] = !collapsed[collection.name])}
          >
            <span class="chev">{collapsed[collection.name] ? "▸" : "▾"}</span>
            <span class="collection-name">{collection.name}</span>
            <span class="count">{collection.entries.length}</span>
            <button
              class="row-action"
              title="Delete collection"
              onclick={(e) => {
                e.stopPropagation();
                deleteCollection(collection.name);
              }}>✕</button
            >
          </div>
          {#if !collapsed[collection.name]}
            {#each visible as entry (entry.rel)}
              {@render entryRow(entry)}
            {/each}
            {#if visible.length === 0}
              <p class="empty-note">
                {collection.entries.length === 0 ? "empty" : "no matches"}
              </p>
            {/if}
          {/if}
        </div>
      {/each}

      {#if app.libraryTree.entries.length === 0 && app.libraryTree.collections.length === 0}
        <div class="empty">
          <p>No saved rules yet.</p>
          <p class="hint">⌘S / Ctrl+S saves the current editor content here.</p>
        </div>
      {/if}
    {/if}
  </div>
</aside>

<style>
  aside {
    width: 250px;
    flex-shrink: 0;
    background: var(--bg1);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    user-select: none;
  }

  header {
    display: flex;
    align-items: center;
    padding: 10px 12px 6px;
  }

  .title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
  }

  .spacer {
    flex: 1;
  }

  .icon {
    background: none;
    border: none;
    color: var(--muted);
    font-size: 14px;
    cursor: pointer;
    padding: 2px 5px;
    border-radius: 4px;
    line-height: 1;
  }
  .icon:hover {
    color: var(--text);
    background: var(--bg2);
  }

  .search {
    margin: 2px 10px 8px;
    background: var(--bg0);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-size: 12px;
    font-family: var(--font-ui);
    padding: 5px 9px;
    outline: none;
  }
  .search:focus {
    border-color: #2e3a50;
  }
  .search::placeholder {
    color: #4a5568;
  }

  .new-collection input {
    margin: 0 10px 8px;
    width: calc(100% - 20px);
    background: var(--bg0);
    border: 1px solid var(--accent);
    border-radius: 6px;
    color: var(--text);
    font-size: 12px;
    font-family: var(--font-ui);
    padding: 5px 9px;
    outline: none;
  }

  .tree {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 12px;
  }

  .collection-head {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 12px;
    cursor: pointer;
    font-size: 12.5px;
    color: var(--text);
  }
  .collection-head:hover {
    background: var(--bg2);
  }

  .chev {
    color: var(--muted);
    font-size: 10px;
    width: 10px;
  }

  .collection-name {
    font-weight: 600;
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .count {
    color: var(--muted);
    font-size: 11px;
  }

  .entry {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px 4px 26px;
    cursor: pointer;
    font-size: 12.5px;
    min-width: 0;
  }
  .entry:hover {
    background: var(--bg2);
  }
  .entry.active {
    background: #1c2a45;
  }

  .entry-name {
    font-family: var(--font-mono);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .broken {
    color: var(--red);
    margin-right: 2px;
  }

  .entry-tags {
    display: flex;
    gap: 3px;
    flex-shrink: 0;
    margin-left: auto;
  }

  .tag {
    font-family: var(--font-mono);
    font-size: 9.5px;
    color: #56d4dd;
    background: #56d4dd15;
    padding: 0 4px;
    border-radius: 3px;
  }

  .row-action {
    background: none;
    border: none;
    color: transparent;
    font-size: 11px;
    cursor: pointer;
    padding: 0 2px;
    flex-shrink: 0;
  }
  .entry:hover .row-action,
  .collection-head:hover .row-action {
    color: var(--muted);
  }
  .row-action:hover {
    color: var(--red) !important;
  }

  .empty-note {
    color: #4a5568;
    font-size: 11px;
    margin: 2px 0 4px 26px;
  }

  .empty {
    padding: 24px 16px;
    text-align: center;
    color: var(--muted);
    font-size: 12.5px;
  }
  .empty .hint {
    color: #4a5568;
    font-size: 11.5px;
  }
</style>
