<script setup lang="ts">
import { computed, inject } from 'vue';
import { encodeLink, type GlobalProps, type Item, type LinkZip } from '@/common';
import LinkItem from './LinkItem.vue';

const props = defineProps<{ linkZips: LinkZip[]; }>();

const gProps = inject<GlobalProps>('props')!;

const srcItem = inject<Item>('srcItem')!;

const filteredLinkZips = computed(() => {
  const out = [];
  const links = gProps.searchResult.value?.links;
  for (const linkZip of props.linkZips) {
    let filtered = true;
    const items = linkZip.codes
      .map(it => {
        let src = srcItem.code, dst = it.code;
        if (linkZip.rev) [src, dst] = [dst, src];

        const enabled = src < 100000 || links == undefined ||
          links.has(encodeLink(src, dst, linkZip.time));
        if (enabled) filtered = false;

        const item = gProps.resolve(it.code, linkZip.time - (linkZip.rev ? 1 : 0))!;
        return { item, desc: it.desc, showDesc: it.showDesc, enabled };
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
          :desc="it.desc != undefined ? [linkZip.time, it.desc] : undefined"
          :show-desc="it.showDesc" />
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
