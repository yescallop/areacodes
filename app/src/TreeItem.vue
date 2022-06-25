<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps(['item']);
const isFolder = computed(() => {
  return props.item.children != undefined;
});
const successors = computed(() => {
  let sus: any[] | undefined = props.item.successors;
  if (sus == undefined) {
    return null;
  }
  let time_or_default = (su: any) => {
    return su.time ? su.time : props.item.end;
  };

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
});
const isOpen = ref(false);

function toggle() {
  if (isFolder.value) isOpen.value = !isOpen.value;
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
    <ul v-if="successors" class="successors">
      <li v-for="su in successors">=> {{ su.codes }} &lt;{{ su.time }}&gt;</li>
    </ul>
    <ul v-if="isOpen">
      <TreeItem v-for="child in item.children" :item="child"></TreeItem>
    </ul>
  </li>
</template>

<style scoped>
ul {
  list-style-type: none;
  padding-left: 2ch;
}

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

.successors {
  color: green;
  padding-left: 5ch;
}
</style>