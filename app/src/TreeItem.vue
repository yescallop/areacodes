<script setup lang="ts">
import { computed, inject, ref } from 'vue';
import { type GlobalProps, type Item, time_or_default } from './common';

const props = defineProps<{ item: Item; open: boolean; }>();
const gProps = inject<GlobalProps>('props')!;

interface LinkWithDirection {
  code: number,
  time: number,
  rev: boolean,
}

interface LinkZipped {
  codes: string,
  time: number,
  rev: boolean,
}

const isFolder = computed(() => props.item.children != undefined);
const isOpen = ref(props.open);
const links = computed(() => {
  return zip_links(get_links());
});

function get_links(): LinkWithDirection[] {
  let item = props.item;
  let predecessors = gProps.predecessors.value.get(item.code);
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
    return diff != 0 ? diff : ((a.rev ? 1 : 0) - (b.rev ? 1 : 0));
  });
  return links;
}

function zip_links(links: LinkWithDirection[]): LinkZipped[] {
  if (links.length == 0) {
    return [];
  }
  let out = [];
  let codes = String(links[0].code);
  let last = links[0];
  links.slice(1).forEach(link => {
    if (link.time == last.time && link.rev == last.rev) {
      codes += "," + link.code;
    } else {
      out.push({ codes, time: last.time, rev: last.rev });
      codes = String(link.code);
      last = link;
    }
  });
  out.push({ codes, time: last.time, rev: last.rev });
  return out;
}
</script>

<template>
  <li>
    <div :class="{ obsolete: item.end, leaf: !isFolder }">
      <span v-if="isFolder" class="toggle" @click="isOpen = !isOpen">[{{ isOpen ? '-' : '+' }}]</span>{{
          item.code
      }}
      &lt;{{ item.start }}{{ item.end ? "-" + item.end : "" }}&gt;
      {{ item.name }}
    </div>
    <ul v-if="links.length" class="links">
      <li v-for="link in links" :class="{ rev: link.rev }">{{ link.codes }} &lt;{{ link.time }}&gt;</li>
    </ul>
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

.links {
  color: green;
  padding-left: 5ch;
}

.links li.rev {
  color: darkred;
}

.links li::before {
  content: "=>";
  padding-right: 1ch;
}

.links li.rev::before {
  content: "<=";
}
</style>