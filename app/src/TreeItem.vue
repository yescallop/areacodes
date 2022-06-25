<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps(['item']);
const isFolder = computed(() => {
  return props.item.children != undefined;
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
    <ul v-if="isOpen">
      <TreeItem v-for="child in item.children" :item="child"></TreeItem>
    </ul>
  </li>
</template>

<style>
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
</style>