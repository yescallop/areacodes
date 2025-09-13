<script setup lang="ts">
import { computed, inject } from 'vue';
import type { GlobalProps, LinkZip } from '@/common';
import LinkItem from './LinkItem.vue';

const props = defineProps<{ linkZips: LinkZip[]; }>();

const gProps = inject<GlobalProps>('props')!;

const filteredLinkZips = computed(() => {
  const out = [];
  for (const linkZip of props.linkZips) {
    const items = linkZip.codes
      .map(it => {
        const item = gProps.resolveLink(it.code, linkZip.time, linkZip.rev);
        return { item, desc: it.desc };
      }).filter(it => {
        if (it.item.code < 100000) return true;
        const res = gProps.searchResult.value;
        return res == undefined || res.has(it.item);
      });
    if (items.length)
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
        <template v-if="index != 0">,</template>
        <LinkItem :item="it.item" :desc="it.desc" />
      </template>
      &lt;{{ linkZip.time }}&gt;
    </li>
  </ul>
</template>

<style>
.links {
  color: green;
  padding-left: 5ch;
}

.links li.rev {
  color: darkred;
}
</style>
