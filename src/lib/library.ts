import { ask } from "@tauri-apps/plugin-dialog";
import {
  libraryCreateCollection,
  libraryDelete,
  libraryDeleteCollection,
  libraryList,
  libraryRead,
  librarySave,
} from "./api";
import { app, NEW_RULE } from "./state.svelte";

export async function refreshLibrary() {
  try {
    app.libraryTree = await libraryList();
    const existing = new Set([
      ...app.libraryTree.entries.map((e) => e.rel),
      ...app.libraryTree.collections.flatMap((c) => c.entries.map((e) => e.rel)),
    ]);
    for (const rel of [...app.scanSet]) {
      if (!existing.has(rel)) app.scanSet.delete(rel);
    }
  } catch (e) {
    app.showFlash(String(e));
  }
}

async function confirmDiscard(): Promise<boolean> {
  if (!app.dirty) return true;
  return ask("The current rule has unsaved changes. Discard them?", {
    title: "Unsaved changes",
    kind: "warning",
  });
}

export async function loadEntry(rel: string) {
  if (!(await confirmDiscard())) return;
  try {
    const source = await libraryRead(rel);
    app.setEditorContent(source);
    app.currentRel = rel;
    app.savedSource = source;
  } catch (e) {
    app.showFlash(String(e));
  }
}

export function splitRel(rel: string): { collection: string | null; name: string } {
  const slash = rel.indexOf("/");
  if (slash === -1) return { collection: null, name: rel };
  return { collection: rel.slice(0, slash), name: rel.slice(slash + 1) };
}

export async function saveAs(collection: string | null, name: string) {
  const rel = await librarySave(collection, name, app.source);
  app.currentRel = rel;
  app.savedSource = app.source;
  app.showFlash("Saved");
  await refreshLibrary();
}

export async function saveCurrent() {
  if (app.currentRel === null) {
    app.saveDialogOpen = true;
    return;
  }
  const { collection, name } = splitRel(app.currentRel);
  try {
    await saveAs(collection, name);
  } catch (e) {
    app.showFlash(String(e));
  }
}

export async function newRule() {
  if (!(await confirmDiscard())) return;
  app.setEditorContent(NEW_RULE);
  app.currentRel = null;
  app.savedSource = null;
}

export async function deleteEntry(rel: string) {
  const yes = await ask(`Delete ${rel} from the library?`, {
    title: "Delete rule",
    kind: "warning",
  });
  if (!yes) return;
  try {
    await libraryDelete(rel);
    if (app.currentRel === rel) {
      app.currentRel = null;
      app.savedSource = null;
    }
    await refreshLibrary();
  } catch (e) {
    app.showFlash(String(e));
  }
}

export async function createCollection(name: string) {
  try {
    await libraryCreateCollection(name);
    await refreshLibrary();
  } catch (e) {
    app.showFlash(String(e));
  }
}

export async function deleteCollection(name: string) {
  const yes = await ask(
    `Delete collection "${name}" and every rule inside it?`,
    { title: "Delete collection", kind: "warning" },
  );
  if (!yes) return;
  try {
    await libraryDeleteCollection(name);
    if (app.currentRel?.startsWith(`${name}/`)) {
      app.currentRel = null;
      app.savedSource = null;
    }
    await refreshLibrary();
  } catch (e) {
    app.showFlash(String(e));
  }
}
