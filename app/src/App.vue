<script setup lang="ts">
import { ref } from 'vue';
import TreeItem from './TreeItem.vue';
import codesUrl from '../../codes.json?url';

const codes = ref([]);
fetch(codesUrl)
  .then(resp => resp.json())
  .then(json => codes.value = json);
</script>

<template>
  <ul>
    <TreeItem class="item" v-for="child in codes" :item="child"></TreeItem>
  </ul>
</template>
