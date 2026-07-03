<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    testsAddSample,
    testsList,
    testsRemoveSample,
    testsRun,
    type EntryTestReport,
    type TestKind,
    type TestSamples,
  } from "$lib/api";
  import { humanSize } from "$lib/format";
  import { app } from "$lib/state.svelte";

  let samples = $state<TestSamples | null>(null);
  let report = $state<EntryTestReport | null>(null);
  let running = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (app.testsRel) {
      report = null;
      error = null;
      refresh(app.testsRel);
    }
  });

  async function refresh(rel: string) {
    try {
      samples = await testsList(rel);
    } catch (e) {
      error = String(e);
    }
  }

  function close() {
    app.testsRel = null;
    samples = null;
  }

  async function addSamples(kind: TestKind) {
    if (!app.testsRel) return;
    const selection = await open({
      multiple: true,
      title:
        kind === "match"
          ? "Samples this rule MUST match"
          : "Samples this rule must NOT match",
    });
    if (!selection) return;
    try {
      for (const path of selection) {
        await testsAddSample(app.testsRel, kind, path);
      }
      report = null;
      await refresh(app.testsRel);
    } catch (e) {
      error = String(e);
    }
  }

  async function removeSample(kind: TestKind, fileName: string) {
    if (!app.testsRel) return;
    try {
      await testsRemoveSample(app.testsRel, kind, fileName);
      report = null;
      await refresh(app.testsRel);
    } catch (e) {
      error = String(e);
    }
  }

  async function run() {
    if (!app.testsRel) return;
    running = true;
    error = null;
    try {
      const full = await testsRun([app.testsRel]);
      report = full.entries[0] ?? null;
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }

  function resultFor(kind: TestKind, fileName: string) {
    return report?.results.find((r) => r.kind === kind && r.fileName === fileName);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={app.testsRel ? onKeydown : undefined} />

{#snippet sampleList(kind: TestKind, list: { fileName: string; size: number }[], title: string, hint: string)}
  <section>
    <header>
      <h3>{title}</h3>
      <button class="ghost small" onclick={() => addSamples(kind)}>+ Add files</button>
    </header>
    {#if list.length === 0}
      <p class="hint">{hint}</p>
    {:else}
      <ul>
        {#each list as sample (sample.fileName)}
          {@const r = resultFor(kind, sample.fileName)}
          <li>
            {#if r}
              <span class="verdict" class:pass={r.passed} class:fail={!r.passed}>
                {r.passed ? "✓" : "✗"}
              </span>
            {:else}
              <span class="verdict idle">·</span>
            {/if}
            <span class="sample-name" title={sample.fileName}>{sample.fileName}</span>
            <span class="size">{humanSize(sample.size)}</span>
            {#if r && !r.passed}
              <span class="why">
                {r.error ??
                  (kind === "match" ? "no rule matched" : `matched: ${r.matchedRules.join(", ")}`)}
              </span>
            {/if}
            <button
              class="row-action"
              title="Remove sample"
              onclick={() => removeSample(kind, sample.fileName)}>✕</button
            >
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/snippet}

{#if app.testsRel}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <div class="dialog" aria-label="Rule regression tests">
      <h2>
        Tests — <code>{app.testsRel}</code>
      </h2>
      <p class="explain">
        Samples are copied into the library next to the rule. Every run recompiles the
        rule and checks each sample against the expectation.
      </p>

      {#if error}
        <p class="error">{error}</p>
      {/if}
      {#if report?.compileError}
        <p class="error">Rule does not compile: {report.compileError}</p>
      {/if}

      {#if samples}
        {@render sampleList(
          "match",
          samples.expectMatch,
          "Must match",
          "No positive samples yet — add files this rule is supposed to detect.",
        )}
        {@render sampleList(
          "no-match",
          samples.expectNoMatch,
          "Must not match",
          "No negative samples yet — add known-clean files to catch false positives.",
        )}
      {/if}

      <div class="buttons">
        {#if report && !report.compileError}
          <span class="run-summary" class:ok={report.failed === 0} class:bad={report.failed > 0}>
            {report.failed === 0
              ? `✓ all ${report.passed} passed`
              : `✗ ${report.failed} failed, ${report.passed} passed`}
          </span>
        {/if}
        <span class="spacer"></span>
        <button class="ghost" onclick={close}>Close</button>
        <button
          class="primary"
          onclick={run}
          disabled={running ||
            !samples ||
            (samples.expectMatch.length === 0 && samples.expectNoMatch.length === 0)}
        >
          {running ? "Running…" : "Run tests"}
        </button>
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
    width: 520px;
    max-height: 80vh;
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
  h2 code {
    font-family: var(--font-mono);
    color: var(--accent);
    font-weight: 500;
  }

  .explain {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }

  section header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }

  h3 {
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--muted);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  li {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
    padding: 3px 6px;
    border-radius: 5px;
    min-width: 0;
  }
  li:hover {
    background: var(--bg1);
  }

  .verdict {
    width: 14px;
    text-align: center;
    font-weight: 700;
    flex-shrink: 0;
  }
  .verdict.pass {
    color: var(--green);
  }
  .verdict.fail {
    color: var(--red);
  }
  .verdict.idle {
    color: var(--muted);
  }

  .sample-name {
    font-family: var(--font-mono);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .size {
    color: var(--muted);
    font-size: 11px;
    flex-shrink: 0;
  }

  .why {
    color: #ffb3ae;
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row-action {
    margin-left: auto;
    background: none;
    border: none;
    color: transparent;
    cursor: pointer;
    font-size: 11px;
    flex-shrink: 0;
  }
  li:hover .row-action {
    color: var(--muted);
  }
  .row-action:hover {
    color: var(--red) !important;
  }

  .hint {
    margin: 0;
    color: #4a5568;
    font-size: 12px;
  }

  .error {
    margin: 0;
    color: #ffb3ae;
    font-size: 12px;
  }

  .buttons {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }

  .run-summary {
    font-size: 12.5px;
    font-weight: 600;
  }
  .run-summary.ok {
    color: var(--green);
  }
  .run-summary.bad {
    color: var(--red);
  }

  .spacer {
    flex: 1;
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
    padding: 2px 8px;
    font-size: 11px;
  }
  .ghost:hover {
    color: var(--text);
  }
</style>
