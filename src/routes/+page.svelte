<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { scanPaths } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import Editor from "$lib/components/Editor.svelte";
  import ResultsPanel from "$lib/components/ResultsPanel.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import DropOverlay from "$lib/components/DropOverlay.svelte";

  let editorWidth = $state(55);
  let main: HTMLElement;

  async function runScan(paths: string[]) {
    if (paths.length === 0) return;
    app.scanning = true;
    app.scanError = null;
    try {
      app.report = await scanPaths(app.source, paths);
    } catch (e) {
      app.scanError = String(e);
    } finally {
      app.scanning = false;
    }
  }

  async function pickFiles() {
    const selection = await open({ multiple: true, title: "Choose files to scan" });
    if (selection) runScan(selection);
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "enter" || payload.type === "over") {
          app.dragActive = true;
        } else if (payload.type === "leave") {
          app.dragActive = false;
        } else if (payload.type === "drop") {
          app.dragActive = false;
          runScan(payload.paths);
        }
      })
      .then((fn) => (unlisten = fn));
    return () => unlisten?.();
  });

  function startResize(event: PointerEvent) {
    event.preventDefault();
    const rect = main.getBoundingClientRect();
    const move = (e: PointerEvent) => {
      const percent = ((e.clientX - rect.left) / rect.width) * 100;
      editorWidth = Math.min(75, Math.max(30, percent));
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }
</script>

<div class="shell">
  <header class="topbar">
    <div class="brand">
      <svg viewBox="0 0 24 24" class="logo" aria-hidden="true">
        <path d="M12 2 21 7v10l-9 5-9-5V7z" fill="none" stroke="var(--accent)" stroke-width="1.6" />
        <path d="M8.5 9 12 12.5 15.5 9M12 12.5V16" fill="none" stroke="var(--accent)" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <h1>YARA Studio</h1>
      <span class="engine">YARA-X</span>
    </div>
    <div class="actions">
      <button
        class="scan"
        onclick={pickFiles}
        disabled={app.scanning}
        title="Scan files against the current rules"
      >
        Scan files…
      </button>
    </div>
  </header>

  <main bind:this={main}>
    <section class="editor-pane" style:width="{editorWidth}%">
      <Editor />
    </section>
    <div
      class="divider"
      role="separator"
      aria-orientation="vertical"
      onpointerdown={startResize}
    ></div>
    <section class="results-pane">
      <ResultsPanel onPickFiles={pickFiles} />
    </section>
  </main>

  <StatusBar />
</div>

<DropOverlay />

<style>
  .shell {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 46px;
    padding: 0 14px;
    background: var(--bg1);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    user-select: none;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
  }

  .logo {
    width: 22px;
    height: 22px;
  }

  h1 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }

  .engine {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--accent);
    background: #e8b33915;
    border: 1px solid #e8b33933;
    padding: 1px 7px;
    border-radius: 10px;
  }

  .scan {
    background: var(--accent);
    color: #1a1405;
    border: none;
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 12.5px;
    font-weight: 600;
    font-family: var(--font-ui);
    cursor: pointer;
  }
  .scan:hover:not(:disabled) {
    background: #f2c14e;
  }
  .scan:disabled {
    opacity: 0.5;
    cursor: default;
  }

  main {
    flex: 1;
    display: flex;
    min-height: 0;
  }

  .editor-pane {
    min-width: 0;
    height: 100%;
  }

  .divider {
    width: 5px;
    cursor: col-resize;
    background: var(--border);
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .divider:hover {
    background: var(--accent);
  }

  .results-pane {
    flex: 1;
    min-width: 0;
    height: 100%;
  }
</style>
