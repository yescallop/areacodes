<script setup lang="ts">
import { inject } from 'vue';
import { type Item, scrollToItem, Action, type GlobalProps } from '@/common';

defineProps<{
  item: Item;
  enabled?: boolean;
  desc?: [number, number];
  showDesc?: boolean;
}>();

const gProps = inject<GlobalProps>('props')!;

const srcItem = inject<Item>('srcItem')!;

function onKeyDown(e: KeyboardEvent) {
  if (e.code == "Backspace") {
    srcItem.action = Action.Focus;
    srcItem.act!();
  }
}
</script>

<template>
  <a :class="{ disabled: !enabled }" href="javascript:"
    @click.prevent="if (enabled) { scrollToItem(item); }" @keydown="onKeyDown">
    <ruby v-if="item.name != srcItem.name">{{ item.code }}<rt>{{
      item.name }}</rt></ruby>
    <template v-else>{{ item.code }}</template>
  </a>
  <a v-if="showDesc && !gProps.searchResult.value?.desc" href="javascript:" class="desc"
    @click="gProps.options.searchText = `${desc![0]}.${desc![1] + 1}`">[?]</a>
</template>

<style scoped>
rt {
  text-align: center;
  font-size: 75%;

  @supports (-moz-appearance: none) {
    margin-bottom: -6px;
  }

  @supports not (-moz-appearance: none) {
    margin: 0 5.5px;
  }
}

.desc {
  user-select: none;
  color: darkblue;
}

.disabled {
  color: gray;
}
</style>
