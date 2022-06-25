<script setup lang="ts">
import { ref } from 'vue';

interface Successor {
  opt?: boolean,
  time?: number,
  code: number,
}

interface Item {
  code: number,
  name: string,
  start: number,
  end?: number,
  sus?: Successor[],
  children?: Item[],
}

interface Su {
  codes: string,
  time: number,
}

const props = defineProps<{
  item: Item,
}>();
const isFolder = props.item.children != undefined;
const isOpen = ref(false);
const sus = get_sus();

function get_sus(): { non_opt: Su[], opt: Su[]; } {
  let sus = props.item.sus != undefined ? props.item.sus : [];
  let sus_opt: Successor[] = [];
  let i = sus.findIndex(su => su.opt);
  if (i >= 0) {
    sus_opt = sus.slice(i);
    sus = sus.slice(0, i);
  }
  return { non_opt: zip_sus(sus), opt: zip_sus(sus_opt) };
}

function zip_sus(sus: Successor[]): Su[] {
  if (sus.length == 0) {
    return [];
  }
  let out = [];
  let codes = String(sus[0].code);
  let lastTime = time_or_default(sus[0]);
  sus.slice(1).forEach(su => {
    let time = time_or_default(su);
    if (time == lastTime) {
      codes += "," + su.code;
    } else {
      out.push({ codes, time: lastTime });
      codes = String(su.code);
      lastTime = time;
    }
  });
  out.push({ codes, time: lastTime });
  return out;
}

function time_or_default(su: Successor): number {
  return su.time ? su.time : props.item.end!;
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
    <ul v-if="sus.non_opt.length" class="sus_non_opt">
      <li v-for="su in sus.non_opt">=> {{ su.codes }} &lt;{{ su.time }}&gt;</li>
    </ul>
    <ul v-if="sus.opt.length" class="sus_opt">
      <li v-for="su in sus.opt">~> {{ su.codes }} &lt;{{ su.time }}&gt;</li>
    </ul>
    <ul v-if="isOpen">
      <TreeItem v-for="child in item.children" :item="child"></TreeItem>
    </ul>
  </li>
</template>

<style scoped>
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

.sus_non_opt {
  color: green;
  padding-left: 5ch;
}

.sus_opt {
  color: brown;
  padding-left: 5ch;
}
</style>