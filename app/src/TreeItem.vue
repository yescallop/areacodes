<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps(['item']);
const isFolder = computed(() => {
  return props.item.children && props.item.children.length;
});
const isOpen = ref(false);

function toggle() {
  if (isFolder.value) isOpen.value = !isOpen.value;
}
</script>

<template>
  <li>
    <div :class="{ obsolete: item.end }" @click="toggle">
      {{ item.code }} {{ item.name }}
      <span v-if="isFolder">[{{ isOpen ? '-' : '+' }}]</span>
    </div>
    <ul v-show="isOpen" v-if="isFolder">
      <TreeItem class="item" v-for="child in item.children" :item="child"></TreeItem>
    </ul>
  </li>
</template>

<style>
.obsolete {
  color: gray;
}
</style>