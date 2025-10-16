<!-- eslint-disable vue/no-mutating-props -->
<script setup lang="ts">
import { computed, inject, nextTick, onMounted, onUnmounted, onUpdated, provide, ref, toRaw, watch } from 'vue';
import type { GlobalProps, Item, LinkCode, LinkZip } from '@/common';
import { timeOrDefault, Action, inUse, exposeItem } from '@/common';
import LinkGroup from './LinkGroup.vue';

const props = defineProps<{ item: Item; }>();
const gProps = inject<GlobalProps>('props')!;

provide('srcItem', props.item);

const isFolder = computed(() => children.value.length > 0);
const isOpen = ref(props.item.root ?? false);
const linkZips = computed(() => zipLinks(getLinks()));
const headLink = ref<HTMLElement>();

const children = computed(() => props.item.children?.filter(item => {
  if (item.code < 100000) return true;
  const res = gProps.searchResult.value;
  return res == undefined || res.items.has(toRaw(item));
}) ?? []);

const isHit = computed(() => {
  const res = gProps.searchResult.value;
  return res != undefined && res.time == undefined &&
    res.hits?.has(toRaw(props.item));
});

if (props.item.guide) {
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
    if (action & Action.Open) {
      isOpen.value = isFolder.value;
    } else if (action & Action.Close) {
      isOpen.value = false;
    }

    const scroll = (action & Action.Scroll) != 0;
    if (action & Action.Focus)
      headLink.value?.focus({ preventScroll: scroll });
    if (scroll) document.fonts.ready.then(() => {
      headLink.value?.scrollIntoView();
    });
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
    const parent = props.item.parent;
    if (parent != undefined) {
      exposeItem(parent, Action.Focus);
    }
  } else if (e.code == "Minus") {
    isOpen.value = false;
  } else if (e.code == "Equal") {
    isOpen.value = true;
  }
}

function onClick() {
  const item = props.item;
  gProps.pushHistory(item);
  gProps.options.searchText =
    `${item.code},${item.start}-${item.end ?? ''}`;
  nextTick(() => exposeItem(item, Action.Scroll));
}
</script>

<template>
  <li>
    <div v-if="!item.root" :class="{ obsolete: item.end, leaf: !isFolder, hit: isHit }">
      <span v-if="isFolder" class="toggle" @click="isOpen = !isOpen">[{{ isOpen ? '-' : '+' }}]</span>
      <a ref="headLink" href="javascript:"
        @click="onClick" @keydown="onKeyDown">{{ item.code }} {{ item.name }}
        <template v-if="item.start">{{ ` (${item.start}-${item.end ?? ''})` }}</template>
      </a>
    </div>
    <LinkGroup :link-zips="linkZips" />
    <ul v-if="isOpen">
      <TreeItem v-for="it in children" :item="it" :key="it.code * 10000 + it.start" />
    </ul>
  </li>
</template>

<style scoped>
div {
  text-indent: -4ch;
  margin-left: 4ch;
}

.hit>a {
  font-weight: 600;
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
</style>
