<script lang="ts">
  import { app } from "$lib/state.svelte";
</script>

{#if app.dragActive}
  <div class="overlay">
    <div class="frame">
      <svg viewBox="0 0 64 64" aria-hidden="true">
        <path d="M32 14v26m0 0-9-9m9 9 9-9" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M16 48h32" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
      </svg>
      <p>Release to scan</p>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: #0a0e14cc;
    backdrop-filter: blur(2px);
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .frame {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 48px 72px;
    border: 2px dashed var(--accent);
    border-radius: 14px;
    color: var(--accent);
    background: #e8b33908;
  }

  svg {
    width: 56px;
    height: 56px;
  }

  p {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
  }
</style>
