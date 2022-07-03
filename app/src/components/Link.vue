<script setup lang="ts">
import { computed, inject } from 'vue';
import { type Item, type GlobalProps, scrollToItem, Action } from '@/common';

const props = defineProps<{ code: number, time: number, rev: boolean; }>();
const gProps = inject<GlobalProps>('props')!;
const srcItem = inject<Item>('srcItem')!;
const item = computed(() => {
  let time = props.time - (props.rev ? 1 : 0);
  // The items are by default descending in time.
  return gProps.items.get(props.code)!.find(item => time >= item.start)!;
});

function onKeyDown(e: KeyboardEvent) {
  if (e.code == "Backspace") {
    srcItem.action = Action.Focus;
    srcItem.act!();
  }
}
</script>

<template>
  <a :href="`#${item.code}:${item.start}`" @click.prevent="scrollToItem(item)" @keydown="onKeyDown">
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
  margin-bottom: -6px;
}
</style>
