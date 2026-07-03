<script lang="ts">
  import { saveAs } from "$lib/library";
  import { app } from "$lib/state.svelte";

  let name = $state("");
  let collection = $state<string>("");
  let newCollection = $state("");
  let error = $state<string | null>(null);

  const collections = $derived(app.libraryTree?.collections.map((c) => c.name) ?? []);

  const suggestedName = $derived.by(() => {
    const m = app.source.match(/^\s*(?:private\s+|global\s+)*rule\s+([A-Za-z0-9_]+)/m);
    return m ? m[1] : "";
  });

  $effect(() => {
    if (app.saveDialogOpen) {
      name = suggestedName;
      error = null;
    }
  });

  function close() {
    app.saveDialogOpen = false;
  }

  async function submit(e: Event) {
    e.preventDefault();
    const target =
      collection === "__new__" ? newCollection.trim() : collection || null;
    if (collection === "__new__" && !target) {
      error = "Collection name is empty";
      return;
    }
    if (name.trim() === "") {
      error = "Rule file name is empty";
      return;
    }
    try {
      await saveAs(target || null, name.trim());
      close();
    } catch (err) {
      error = String(err);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={app.saveDialogOpen ? onKeydown : undefined} />

{#if app.saveDialogOpen}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <form class="dialog" aria-label="Save rule to library" onsubmit={submit}>
      <h2>Save to library</h2>

      <label>
        <span>File name</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input autofocus bind:value={name} placeholder="my_rule" spellcheck="false" />
        <span class="ext-hint">.yar is added automatically</span>
      </label>

      <label>
        <span>Collection</span>
        <select bind:value={collection}>
          <option value="">— library root —</option>
          {#each collections as c}
            <option value={c}>{c}</option>
          {/each}
          <option value="__new__">new collection…</option>
        </select>
      </label>

      {#if collection === "__new__"}
        <label>
          <span>New collection name</span>
          <input bind:value={newCollection} placeholder="apt" spellcheck="false" />
        </label>
      {/if}

      {#if error}
        <p class="error">{error}</p>
      {/if}

      <div class="buttons">
        <button type="button" class="ghost" onclick={close}>Cancel</button>
        <button type="submit" class="primary">Save</button>
      </div>
    </form>
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
    width: 340px;
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: 0 12px 40px #00000066;
  }

  h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-size: 12px;
    color: var(--muted);
  }

  input,
  select {
    background: var(--bg0);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-size: 13px;
    font-family: var(--font-mono);
    padding: 6px 10px;
    outline: none;
  }
  input:focus,
  select:focus {
    border-color: var(--accent);
  }

  .ext-hint {
    font-size: 10.5px;
    color: #4a5568;
  }

  .error {
    margin: 0;
    color: #ffb3ae;
    font-size: 12px;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
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
  .primary:hover {
    background: #f2c14e;
  }

  .ghost {
    background: none;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--muted);
    padding: 6px 12px;
    font-size: 12.5px;
    font-family: var(--font-ui);
    cursor: pointer;
  }
  .ghost:hover {
    color: var(--text);
  }
</style>
