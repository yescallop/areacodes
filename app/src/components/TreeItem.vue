<!-- eslint-disable vue/no-mutating-props -->
<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, onUpdated, provide, ref, toRaw, watch } from 'vue';
import type { GlobalProps, Item, LinkCode, LinkZip } from '@/common';
import { timeOrDefault, Action } from '@/common';
import LinkGroup from './LinkGroup.vue';

const props = defineProps<{ item: Item; }>();
const gProps = inject<GlobalProps>('props')!;

provide('srcItem', props.item);

const isFolder = computed(() => props.item.children != undefined);
const isOpen = ref(false);
const isHidden = computed(() => {
  if (props.item.code < 100000) return false;
  const res = gProps.searchResult.value;
  return res != undefined && !res.has(toRaw(props.item));
});
const linkZips = computed(() => zipLinks(getLinks()));
const headLink = ref<HTMLElement>();

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
      const hash = `#${props.item.code}:${props.item.start}`;
      if (hash != location.hash) {
        history.pushState(null, "", hash);
      }

      const open = action == Action.OpenFocusScroll;
      if (isFolder.value && action != Action.Focus) {
        isOpen.value = open;
      }

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
    links = predecessors.filter(link => {
      return link.time! >= item.start && (item.end == undefined || link.time! < item.end);
    }).map(link => {
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
  let codes: LinkCode[] = [{ code: prev.code }];

  links.slice(1).forEach(link => {
    if (link.time == prev.time && link.rev == prev.rev) {
      if (link.desc != prev.desc && prev.desc != undefined) {
        codes[codes.length - 1]!.desc = prev.desc;
      }
      codes.push({ code: link.code });
    } else {
      if (prev.desc != undefined) {
        codes[codes.length - 1]!.desc = prev.desc;
      }
      out.push({ codes, time: prev.time, rev: prev.rev });
      codes = [{ code: link.code }];
    }
    prev = link;
  });
  if (prev.desc != undefined) {
    codes[codes.length - 1]!.desc = prev.desc;
  }
  out.push({ codes, time: prev.time, rev: prev.rev });
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
      const parent = props.item.parent;
      if (parent != undefined) {
        parent.action = Action.Focus;
        parent.act!();
      }
    }
  }
}
</script>

<template>
  <li v-if="!isHidden">
    <div class="item" :class="{ obsolete: item.end, leaf: !isFolder }">
      <span v-if="isFolder" class="toggle" @click="isOpen = !isOpen">[{{ isOpen ? '-' : '+' }}] </span>
      <a ref="headLink" :href="`#${item.code}:${item.start}`" @click.prevent="scrollOpen" @keydown="onKeyDown">{{
        item.code
      }}</a> <template v-if="gProps.options.searchText && item.name.startsWith(gProps.options.searchText)">
        <span class="hit">{{ gProps.options.searchText }}</span>{{
          item.name.substring(gProps.options.searchText.length)
        }}
      </template>
      <template v-else>{{ item.name }}</template>
      <template v-if="item.start">
        &lt;{{ item.start }}{{ item.end ? "-" + item.end : "" }}&gt;
      </template>
    </div>
    <LinkGroup :link-zips="linkZips" />
    <ul v-if="isOpen">
      <TreeItem v-for="it in item.children" :item="it" :key="it.code * 10000 + it.start" />
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
}

.hit {
  text-decoration: underline;
}
</style>
