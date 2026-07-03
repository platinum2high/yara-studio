<script lang="ts">
  import { app } from "$lib/state.svelte";

  function close() {
    app.testsReport = null;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={app.testsReport ? onKeydown : undefined} />

{#if app.testsReport}
  {@const r = app.testsReport}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <div class="dialog" aria-label="Library test report">
      <h2>Library test run</h2>

      <p class="summary" class:ok={r.totalFailed === 0} class:bad={r.totalFailed > 0}>
        {#if r.entries.length === 0}
          No rules have tests yet — open a rule's tests via the flask icon in the library.
        {:else if r.totalFailed === 0}
          ✓ {r.totalPassed} test{r.totalPassed === 1 ? "" : "s"} passed across {r.entries.length}
          rule file{r.entries.length === 1 ? "" : "s"}
        {:else}
          ✗ {r.totalFailed} failed · {r.totalPassed} passed
        {/if}
        {#if r.entriesWithoutTests > 0}
          <span class="skipped">({r.entriesWithoutTests} without tests)</span>
        {/if}
      </p>

      <div class="entries">
        {#each r.entries as entry (entry.rel)}
          <div class="entry" class:failed={entry.failed > 0}>
            <div class="entry-head">
              <span class="verdict">{entry.failed === 0 ? "✓" : "✗"}</span>
              <code>{entry.rel}</code>
              <span class="counts">{entry.passed}/{entry.passed + entry.failed}</span>
            </div>
            {#if entry.compileError}
              <p class="why">{entry.compileError}</p>
            {/if}
            {#each entry.results.filter((x) => !x.passed) as bad}
              <p class="why">
                <b>{bad.fileName}</b> —
                {bad.error ??
                  (bad.kind === "match"
                    ? "expected a match, none fired"
                    : `expected clean, matched: ${bad.matchedRules.join(", ")}`)}
              </p>
            {/each}
          </div>
        {/each}
      </div>

      <div class="buttons">
        <button class="ghost" onclick={close}>Close</button>
      </div>
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
    width: 480px;
    max-height: 75vh;
    overflow-y: auto;
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 12px 40px #00000066;
  }

  h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .summary {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }
  .summary.ok {
    color: var(--green);
  }
  .summary.bad {
    color: var(--red);
  }
  .skipped {
    color: var(--muted);
    font-weight: 400;
    font-size: 12px;
  }

  .entries {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .entry {
    border: 1px solid var(--border);
    border-radius: 7px;
    padding: 7px 10px;
  }
  .entry.failed {
    border-color: #f851493a;
  }

  .entry-head {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
  }
  .entry-head code {
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .entry .verdict {
    font-weight: 700;
    color: var(--green);
  }
  .entry.failed .verdict {
    color: var(--red);
  }
  .counts {
    margin-left: auto;
    color: var(--muted);
    font-size: 11.5px;
    font-family: var(--font-mono);
  }

  .why {
    margin: 4px 0 0 22px;
    color: #ffb3ae;
    font-size: 11.5px;
  }
  .why b {
    font-family: var(--font-mono);
    font-weight: 600;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
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
