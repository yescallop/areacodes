<script setup lang="ts">
import { computed, inject } from 'vue';
import type { GlobalProps, LinkZipped } from '@/common';
import Link from './Link.vue';

const props = defineProps<{ links: LinkZipped[]; }>();

const gProps = inject<GlobalProps>('props')!;

const filteredLinks = computed(() => {
  let out = [];
  for (const link of props.links) {
    let items = link.codes
      .map(code => gProps.resolveLink(code, link.time, link.rev))
      .filter(item => {
        if (item.code < 100000) return true;
        let res = gProps.searchResult.value;
        return res == undefined || res.has(item);
      });
    if (items.length)
      out.push({ items, time: link.time, rev: link.rev });
  }
  return out;
});
</script>
<template>
  <ul v-if="filteredLinks.length" class="links">
    <li v-for="link in filteredLinks" :class="{ rev: link.rev }">
      {{ link.rev ? "<=" : "=>" }}
      <template v-for="(item, index) in link.items">
        <template v-if="index != 0">,</template>
        <Link :item="item" />
      </template>
      &lt;{{ link.time }}&gt;
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