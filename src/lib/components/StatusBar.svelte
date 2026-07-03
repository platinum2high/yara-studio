<script lang="ts">
  import { app } from "$lib/state.svelte";
</script>

<footer>
  {#if app.validation === null}
    <span class="status pending">compiling…</span>
  {:else if app.validation.ok}
    <span class="status ok">
      ✓ {app.validation.ruleCount} rule{app.validation.ruleCount === 1 ? "" : "s"} compiled
    </span>
  {:else}
    <span class="status bad">
      ✗ {app.validation.errors.length} error{app.validation.errors.length === 1 ? "" : "s"}
    </span>
  {/if}
  {#if app.validation && app.validation.warnings.length > 0}
    <span class="status warn">⚠ {app.validation.warnings.length}</span>
  {/if}

  {#if app.flash}
    <span class="flash">{app.flash}</span>
  {/if}

  <span class="spacer"></span>

  <span class="item">Ln {app.cursor.line}, Col {app.cursor.column}</span>
  <span class="sep"></span>
  <span class="item">YARA-X 1.19</span>
  <span class="sep"></span>
  <span class="item offline">● offline</span>
</footer>

<style>
  footer {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 26px;
    padding: 0 14px;
    background: var(--bg1);
    border-top: 1px solid var(--border);
    font-size: 11.5px;
    color: var(--muted);
    flex-shrink: 0;
    user-select: none;
  }

  .status {
    font-weight: 600;
  }
  .status.ok {
    color: var(--green);
  }
  .status.bad {
    color: var(--red);
  }
  .status.warn {
    color: var(--accent);
  }
  .status.pending {
    color: var(--muted);
  }

  .flash {
    color: var(--accent);
    font-weight: 600;
  }

  .spacer {
    flex: 1;
  }

  .sep {
    width: 1px;
    height: 12px;
    background: var(--border);
  }

  .offline {
    color: var(--green);
  }
</style>
