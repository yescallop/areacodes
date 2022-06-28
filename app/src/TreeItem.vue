<script setup lang="ts">
import { computed, inject, ref } from 'vue';
import { type GlobalProps, type Item, type Link, time_or_default } from './common';

const props = defineProps<{ item: Item; }>();
const gProps = inject<GlobalProps>('props')!;

const isFolder = computed(() => props.item.children != undefined);
const isOpen = ref(false);
const links = computed(() => {
  let item = props.item;
  let links: Link[];
  if (!gProps.reversed.value) {
    links = item.successors != undefined ? item.successors : [];
  } else {
    let predecessors = gProps.predecessors.value.get(item.code);
    if (predecessors != undefined) {
      links = predecessors.filter(link =>
        link.time! >= item.start && (item.end == undefined || link.time! < item.end));
      links.sort((a, b) => a.time! - b.time!);
    } else {
      links = [];
    }
  }
  return zip_links(links);
});

function zip_links(links: Link[]): {
  codes: string,
  time: number,
}[] {
  if (links.length == 0) {
    return [];
  }
  let out = [];
  let codes = String(links[0].code);
  let lastTime = time_or_default(links[0], props.item);
  links.slice(1).forEach(link => {
    let time = time_or_default(link, props.item);
    if (time == lastTime) {
      codes += "," + link.code;
    } else {
      out.push({ codes, time: lastTime });
      codes = String(link.code);
      lastTime = time;
    }
  });
  out.push({ codes, time: lastTime });
  return out;
}

function toggle() {
  isOpen.value = !isOpen.value;
}
</script>

<template>
  <li>
    <div :class="{ obsolete: item.end, leaf: !isFolder }">
      <span v-if="isFolder" class="toggle" @click="toggle">[{{ isOpen ? '-' : '+' }}]</span>{{
          item.code
      }}
      &lt;{{ item.start }}{{ item.end ? "-" + item.end : "" }}&gt;
      {{ item.name }}
    </div>
    <ul v-if="links.length" class="links">
      <li v-for="link in links">{{ gProps.reversed.value ? "<=" : "=>" }} {{ link.codes }} &lt;{{ link.time }}&gt;</li>
    </ul>
    <ul v-if="isOpen">
      <TreeItem v-for="child in item.children" :item="child"></TreeItem>
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
  margin-right: 1ch;
}

.links {
  color: green;
  padding-left: 5ch;
}
</style>