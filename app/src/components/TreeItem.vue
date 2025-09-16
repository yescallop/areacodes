<!-- eslint-disable vue/no-mutating-props -->
<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, onUpdated, provide, ref, toRaw, watch } from 'vue';
import type { GlobalProps, Item, LinkCode, LinkZip } from '@/common';
import { timeOrDefault, Action, inUse } from '@/common';
import LinkGroup from './LinkGroup.vue';

const props = defineProps<{ item: Item; }>();
const gProps = inject<GlobalProps>('props')!;

provide('srcItem', props.item);

const isFolder = computed(() => props.item.children != undefined);
const isOpen = ref(props.item.root ?? false);
const linkZips = computed(() => zipLinks(getLinks()));
const headLink = ref<HTMLElement>();

const filteredChildren = computed(() => props.item.children?.filter(item => {
  if (item.code < 100000) return true;
  const res = gProps.searchResult.value;
  return res == undefined || res.items.has(toRaw(item));
}));

if (props.item.code == 0) {
  watch(isOpen, v => v ?
    localStorage.removeItem("closeGuide") :
    localStorage.setItem("closeGuide", "true")
  );
}

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
  const action = props.item.action;
  if (action != undefined) {
    if (action == Action.Open) {
      isOpen.value = isFolder.value;
    } else if (action == Action.Close) {
      isOpen.value = false;
    } else {
      const open = action != Action.Focus;
      if (open && isFolder.value) {
        isOpen.value = true;
      }

      if (action != Action.OpenScroll)
        headLink.value?.focus({ preventScroll: open });
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
  desc?: number,
  rev: boolean,
}

function getLinks(): LinkWithDirection[] {
  const item = props.item;
  const predecessors = gProps.predecessors.get(item.code);
  let links: LinkWithDirection[];
  if (!gProps.options.hidePredecessors && predecessors != undefined) {
    links = predecessors.filter(link => inUse(item, link.time!))
      .map(link => {
        return { code: link.code, time: link.time!, desc: link.desc, rev: true };
      });
  } else {
    links = [];
  }

  if (!gProps.options.hideSuccessors) {
    item.succ?.forEach(link => {
      links.push({
        code: link.code,
        time: timeOrDefault(link, item),
        desc: link.desc,
        rev: false,
      });
    });
  }

  links.sort((a, b) => {
    const diff = a.time - b.time;
    return diff != 0 ? diff : ((b.rev ? 1 : 0) - (a.rev ? 1 : 0));
  });
  return links;
}

function zipLinks(links: LinkWithDirection[]): LinkZip[] {
  if (links.length == 0) {
    return [];
  }
  const out: LinkZip[] = [];
  let prev = links[0]!;
  let codes: LinkCode[] = [{ code: prev.code, desc: prev.desc }];

  links.slice(1).forEach(link => {
    if (link.time == prev.time && link.rev == prev.rev) {
      if (link.desc != prev.desc && prev.desc != undefined) {
        codes[codes.length - 1]!.showDesc = true;
      }
      codes.push({ code: link.code, desc: link.desc });
    } else {
      if (prev.desc != undefined) {
        codes[codes.length - 1]!.showDesc = true;
      }
      out.push({ codes, time: prev.time, rev: prev.rev });
      codes = [{ code: link.code, desc: link.desc }];
    }
    prev = link;
  });
  if (prev.desc != undefined) {
    codes[codes.length - 1]!.showDesc = true;
  }
  out.push({ codes, time: prev.time, rev: prev.rev });
  return out;
}

function onKeyDown(e: KeyboardEvent) {
  if (e.code == "Backspace") {
    if (isOpen.value) {
      isOpen.value = false;
    } else {
      const parent = props.item.parent;
      if (parent != undefined) {
        parent.action = Action.Focus;
        parent.act!();
      }
    }
  } else if (e.code == "Minus") {
    isOpen.value = false;
  } else if (e.code == "Equal") {
    isOpen.value = true;
  }
}
</script>

<template>
  <li>
    <div v-if="!item.root" class="item" :class="{ obsolete: item.end, leaf: !isFolder }">
      <span v-if="isFolder" class="toggle" @click="isOpen = !isOpen">[{{ isOpen ? '-' : '+' }}]</span>
      <a ref="headLink" href="javascript:"
        @click="gProps.options.searchText = `${item.code},${item.start}-${item.end ?? ''}`" @keydown="onKeyDown">{{
          item.code
        }}</a> <template v-if="gProps.options.searchText && item.name.startsWith(gProps.options.searchText)">
        <span class="hit">{{ gProps.options.searchText }}</span>{{
          item.name.substring(gProps.options.searchText.length)
        }}
      </template>
      <template v-else>{{ item.name }}</template>
      <template v-if="item.start">{{ ` (${item.start}-${item.end ?? ''})` }}</template>
    </div>
    <LinkGroup :link-zips="linkZips" />
    <ul v-if="isOpen">
      <TreeItem v-for="it in filteredChildren" :item="it" :key="it.code * 10000 + it.start" />
    </ul>
  </li>
</template>

<style scoped>
.item {
  text-indent: -4ch;
  margin-left: 4ch;
}

.obsolete {
  color: gray;
}

.leaf {
  text-indent: 0;
  margin-left: 4ch;
}

.toggle {
  user-select: none;
  margin-right: 1ch;
}

.hit {
  text-decoration: underline;
}
</style>
