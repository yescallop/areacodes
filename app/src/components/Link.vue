<script setup lang="ts">
import { inject } from 'vue';
import { type Item, scrollToItem, Action } from '@/common';

defineProps<{ item: Item; desc?: string; }>();

const srcItem = inject<Item>('srcItem')!;

function onKeyDown(e: KeyboardEvent) {
  if (e.code == "Backspace") {
    srcItem.action = Action.Focus;
    srcItem.act!();
  }
}
</script>

<template>
  <a :href="`#${item.code}:${item.start}`" :title="desc" @click.prevent="scrollToItem(item)" @keydown="onKeyDown">
    <ruby v-if="item.name != srcItem.name">{{ item.code }}<rt>{{
      item.name
    }}</rt></ruby>
    <template v-else>{{ item.code }}</template>
  </a>
</template>

<style>
rt {
  text-align: center;
  user-select: none;
  font-size: 75%;
  @supports (-moz-appearance: none) {
    margin-bottom: -6px;
  }
}
</style>
