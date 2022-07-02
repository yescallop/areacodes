<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, onUpdated, provide, ref } from 'vue';
import type { GlobalProps, Item, LinkZipped } from '@/common';
import { timeOrDefault, Action } from '@/common';
import Links from './Links.vue';

const props = defineProps<{ item: Item; }>();
const gProps = inject<GlobalProps>('props')!;

provide('srcItem', props.item);

const isFolder = computed(() => props.item.children != undefined);
const isOpen = ref(false);
const links = computed(() => zipLinks(getLinks()));
const headLink = ref<HTMLElement>();

onMounted(() => {
  props.item.act = act;
  act();
});
onUpdated(() => {
  props.item.act = act;
  act();
});
onUnmounted(() => props.item.act = undefined);

function act() {
  let action = props.item.action;
  if (action != undefined) {
    if (action == Action.Open) {
      isOpen.value = true;
    } else {
      let hash = `#${props.item.code}:${props.item.start}`;
      if (hash != location.hash) {
        history.pushState(null, "", hash);
      }

      let open = action == Action.OpenFocusScroll;
      if (isFolder.value && action != Action.Focus) {
        isOpen.value = open;
      }

      headLink.value!.focus({ preventScroll: open });
      if (open) document.fonts.ready.then(() => {
        headLink.value!.scrollIntoView({ behavior: "smooth" });
      });
    }
    props.item.action = undefined;
  }
}

interface LinkWithDirection {
  code: number,
  time: number,
  rev: boolean,
}

function getLinks(): LinkWithDirection[] {
  let item = props.item;
  let predecessors = gProps.predecessors.get(item.code);
  let links: LinkWithDirection[];
  if (!gProps.options.hidePredecessors && predecessors != undefined) {
    links = predecessors.filter(link => {
      return link.time! >= item.start && (item.end == undefined || link.time! < item.end);
    }).map(link => {
      return { code: link.code, time: link.time!, rev: true };
    });
  } else {
    links = [];
  }

  if (!gProps.options.hideSuccessors) {
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

function scrollOpen() {
  props.item.action = Action.OpenFocusScroll;
  act();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.code == "Backspace") {
    if (isOpen.value) {
      isOpen.value = false;
    } else {
      let parent = props.item.parent;
      if (parent != undefined) {
        parent.action = Action.Focus;
        parent.act!();
      }
    }
  }
}
</script>

<template>
  <li>
    <div :class="{ obsolete: item.end, leaf: !isFolder }">
      <span v-if="isFolder" class="toggle" @click="isOpen = !isOpen">[{{ isOpen ? '-' : '+' }}]</span>
      <a ref="headLink" :href="`#${item.code}:${item.start}`" @click.prevent="scrollOpen" @keydown="onKeyDown">{{
          item.code
      }}</a>
      &lt;{{ item.start }}{{ item.end ? "-" + item.end : "" }}&gt;
      {{ item.name }}
    </div>
    <Links :links="links" />
    <ul v-if="isOpen">
      <TreeItem v-for="child in item.children" :item="child" />
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