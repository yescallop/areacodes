<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, onUpdated, ref } from 'vue';
import { type GlobalProps, type Item, type LinkZipped, time_or_default } from '@/common';
import Links from './Links.vue';

const props = defineProps<{ item: Item; open: boolean; }>();
const gProps = inject<GlobalProps>('props')!;

interface LinkWithDirection {
  code: number,
  time: number,
  rev: boolean,
}

const isFolder = computed(() => props.item.children != undefined);
const isOpen = ref(props.open);
const links = computed(() => {
  return zip_links(get_links());
});
const li = ref<HTMLElement | null>(null);

props.item.onSelected = onSelected;
onMounted(onSelected);
onUpdated(onSelected);
onUnmounted(() => props.item.onSelected = undefined);

function onSelected() {
  let sel = props.item.selected;
  if (sel != undefined) {
    if (sel & 1) isOpen.value = true;
    if (sel & 2) document.fonts.ready.then(() => {
      li.value!.scrollIntoView();
    });
    props.item.selected = undefined;
  }
}

function get_links(): LinkWithDirection[] {
  let item = props.item;
  let predecessors = gProps.predecessors.get(item.code);
  let links: LinkWithDirection[];
  if (!gProps.options.hide_pred && predecessors != undefined) {
    links = predecessors.filter(link => {
      return link.time! >= item.start && (item.end == undefined || link.time! < item.end);
    }).map(link => {
      return { code: link.code, time: link.time!, rev: true };
    });
  } else {
    links = [];
  }

  if (!gProps.options.hide_succ) {
    item.successors?.forEach(link => {
      links.push({
        code: link.code,
        time: time_or_default(link, item),
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

function zip_links(links: LinkWithDirection[]): LinkZipped[] {
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
</script>

<template>
  <li :id="`${props.item.code}:${props.item.start}`" ref="li">
    <div :class="{ obsolete: item.end, leaf: !isFolder }">
      <span v-if="isFolder" class="toggle" @click="isOpen = !isOpen">[{{ isOpen ? '-' : '+' }}]</span>
      <a :href="`#${item.code}:${item.start}`">{{ item.code }}</a>
      &lt;{{ item.start }}{{ item.end ? "-" + item.end : "" }}&gt;
      {{ item.name }}
    </div>
    <Links :links="links"></Links>
    <ul v-if="isOpen">
      <TreeItem v-for="child in item.children" :item="child" :open="props.open"></TreeItem>
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
</style>