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
  let out = [];
  let codes = String(sus[0][0]);
  let lastTime = sus[0][1];
  sus.slice(1).forEach(su => {
    if (su[1] == lastTime) {
      codes += "," + su[0];
    } else {
      out.push({ codes, time: lastTime });
      codes = String(su[0]);
      lastTime = su[1];
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
      {{ item.name }}<ul v-if="successors" class="successors">
        <li v-for="su in successors">=> {{ su.codes }} &lt;{{ su.time }}&gt;</li>
      </ul>
    </div>
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

.leaf .successors {
  padding-left: 1ch;
}
</style>