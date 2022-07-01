<script setup lang="ts">
import { computed, inject } from 'vue';
import { type GlobalProps, scrollToItem } from '@/common';

const props = defineProps<{ code: number, time: number, rev: boolean; }>();
const gProps = inject<GlobalProps>('props')!;
const srcName = inject<string>('srcName')!;
const item = computed(() => {
  let time = props.time - (props.rev ? 1 : 0);
  return gProps.items.get(props.code)!.find(item => time >= item.start)!;
});
</script>

<template>
  <a :href="`#${item.code}:${item.start}`" @click.prevent="scrollToItem(item)">
    <ruby v-if="item.name != srcName">{{ item.code }}<rt>{{
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
