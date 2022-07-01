<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, onUpdated, provide, ref } from 'vue';
import type { GlobalProps, Item, LinkZipped } from '@/common';
import { timeOrDefault } from '@/common';
import Links from './Links.vue';

const props = defineProps<{ item: Item; open: boolean; }>();
const gProps = inject<GlobalProps>('props')!;

provide('srcName', props.item.name);

interface LinkWithDirection {
  code: number,
  time: number,
  rev: boolean,
}

const isFolder = computed(() => props.item.children != undefined);
const isOpen = ref(props.open);
const links = computed(() => {
  return zipLinks(getLinks());
});
const headLink = ref<HTMLElement>();

props.item.onSelected = onSelected;
onMounted(onSelected);
onUpdated(onSelected);
onUnmounted(() => props.item.onSelected = undefined);

function onSelected() {
  let sel = props.item.selected;
  if (sel != undefined) {
    if (isFolder.value) isOpen.value = true;
    if (sel != 0) {
      scroll(false);
      headLink.value!.focus({ preventScroll: true });
    }
    props.item.selected = undefined;
  }
}

function scroll(open: boolean) {
  if (open) isOpen.value = true;
  document.fonts.ready.then(() => {
    headLink.value!.scrollIntoView({ behavior: "smooth", inline: "start" });
  });
}

function getLinks(): LinkWithDirection[] {
  let item = props.item;
  let predecessors = gProps.predecessors.get(item.code);
  let links: LinkWithDirection[];
  if (!gProps.options.hidePred && predecessors != undefined) {
    links = predecessors.filter(link => {
      return link.time! >= item.start && (item.end == undefined || link.time! < item.end);
    }).map(link => {
      return { code: link.code, time: link.time!, rev: true };
    });
  } else {
    links = [];
  }

  if (!gProps.options.hideSucc) {
    item.successors?.forEach(link => {
      links.push({
        code: link.code,
        time: timeOrDefault(link, item),
        rev: false,
      });
    });
  }

  links.sort((a, b) => {
    let diff = a.time - b.time;
    return diff != 0 ? diff : ((b.rev ? 1 : 0) - (a.rev ? 1 : 0));
  });
  return links;
}

function zipLinks(links: LinkWithDirection[]): LinkZipped[] {
  if (links.length == 0) {
    return [];
  }
  let out = [];
  let codes = [links[0].code];
  let last = links[0];
  links.slice(1).forEach(link => {
    if (link.time == last.time && link.rev == last.rev) {
      codes.push(link.code);
    } else {
      out.push({ codes, time: last.time, rev: last.rev });
      codes = [link.code];
      last = link;
    }
  });
  out.push({ codes, time: last.time, rev: last.rev });
  return out;
}

function toggle() {
  if (!isOpen.value) {
    scroll(true);
  } else {
    isOpen.value = false;
  }
}
</script>

<template>
  <li>
    <div :class="{ obsolete: item.end, leaf: !isFolder }">
      <span v-if="isFolder" class="toggle" @click="toggle">[{{ isOpen ? '-' : '+' }}]</span>
      <a ref="headLink" :href="`#${item.code}:${item.start}`" @click="toggle">{{ item.code }}</a>
      &lt;{{ item.start }}{{ item.end ? "-" + item.end : "" }}&gt;
      {{ item.name }}
    </div>
    <Links :links="links" />
    <ul v-if="isOpen">
      <TreeItem v-for="child in item.children" :item="child" :open="open" />
    </ul>
  </li>
</template>

<style>
.obsolete {
  color: gray;
}

.leaf {
  margin-left: 4ch;
}

.toggle {
  user-select: none;
  padding-right: 1ch;
}
</style>