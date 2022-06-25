<script setup lang="ts">
import { ref } from 'vue';
import TreeItem from './TreeItem.vue';
import codesUrl from '../../codes.json?url';

const codes = ref<any[]>([]);
fetch(codesUrl)
  .then(resp => resp.json())
  .then(json => codes.value = json);
</script>

<template>
  <main>
    <ul id="top">
      <TreeItem v-for="child in codes" :item="child"></TreeItem>
    </ul>
  </main>
</template>

<style>
@import url('https://fonts.googleapis.com/css2?family=Noto+Sans+SC&family=Roboto+Mono&display=swap');

body {
  font-family: 'Roboto Mono', 'Noto Sans SC';
}

#top {
  list-style-type: none;
  padding-left: 0;
}
</style>