<script setup lang="ts">
import { computed, inject } from 'vue';
import type { GlobalProps, LinkZip } from '@/common';
import LinkItem from './LinkItem.vue';

const props = defineProps<{ linkZips: LinkZip[]; }>();

const gProps = inject<GlobalProps>('props')!;

const filteredLinkZips = computed(() => {
  const out = [];
  const res = gProps.searchResult.value;
  for (const linkZip of props.linkZips) {
    let filtered = true;
    const items = linkZip.codes
      .map(it => {
        const item = gProps.resolveLink(it.code, linkZip.time, linkZip.rev);
        const enabled = item.code < 100000 || res == undefined || res.has(item);
        if (enabled) filtered = false;
        return { item, desc: it.desc, enabled };
      });
    if (!filtered)
      out.push({ items, time: linkZip.time, rev: linkZip.rev });
  }
  return out;
});
</script>
<template>
  <ul v-if="filteredLinkZips.length" class="links">
    <li v-for="linkZip in filteredLinkZips" :class="{ rev: linkZip.rev }" :key="linkZip.time">
      {{ linkZip.rev ? "<=" : "=>" }}
      <template v-for="(it, index) in linkZip.items" :key="it.item.code">
        <template v-if="index != 0">,<wbr /></template>
        <LinkItem :item="it.item" :enabled="it.enabled"
          :desc="it.desc != undefined ? [linkZip.time, it.desc] : undefined" />
      </template>
      &lt;{{ linkZip.time }}&gt;
    </li>
  </ul>
</template>

<style scoped>
.links {
  color: darkgreen;
  margin-left: 7ch;
  text-indent: -3ch;
}

.links li.rev {
  color: darkred;
}
</style>
