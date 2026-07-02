<script lang="ts">
  import type { RuleMatch } from "$lib/api";
  import { hexOffset } from "$lib/format";

  let { size, ruleMatches }: { size: number; ruleMatches: RuleMatch[] } = $props();

  interface Marker {
    left: number;
    width: number;
    label: string;
  }

  const markers = $derived.by(() => {
    if (size === 0) return [] as Marker[];
    const out: Marker[] = [];
    for (const rule of ruleMatches) {
      for (const m of rule.stringMatches) {
        out.push({
          left: (m.offset / size) * 100,
          width: Math.max((m.length / size) * 100, 0.6),
          label: `${rule.identifier} ${m.identifier} @ ${hexOffset(m.offset)}`,
        });
      }
    }
    return out;
  });
</script>

{#if markers.length > 0}
  <div class="stripe" title="Match positions across the file">
    {#each markers as marker}
      <span
        class="mark"
        style:left="{marker.left}%"
        style:width="{marker.width}%"
        title={marker.label}
      ></span>
    {/each}
  </div>
{/if}

<style>
  .stripe {
    position: relative;
    height: 6px;
    background: #161d2b;
    border-radius: 3px;
    overflow: hidden;
  }

  .mark {
    position: absolute;
    top: 0;
    bottom: 0;
    background: var(--accent);
    border-radius: 1px;
    min-width: 2px;
  }
</style>
