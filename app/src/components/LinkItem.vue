<script setup lang="ts">
import { inject } from 'vue';
import { type Item, scrollToItem, Action, type GlobalProps } from '@/common';

const props = defineProps<{
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

function onLinkClick() {
  if (props.enabled) {
    gProps.pushHistory(srcItem);
    scrollToItem(props.item, Action.Focus);
  }
}

function onDescClick() {
  gProps.pushHistory(srcItem);
  const [time, desc] = props.desc!;
  gProps.options.searchText = `${time}.${desc + 1}`;
}
</script>

<template>
  <a :class="{ disabled: !enabled }"
    :href="props.enabled ? 'javascript:' : undefined"
    @click="onLinkClick" @keydown="onKeyDown">
    <ruby v-if="item.name != srcItem.name">{{ item.code }}<rt>{{
      item.name }}</rt></ruby>
    <template v-else>{{ item.code }}</template>
  </a>
  <sup>
    <a v-if="showDesc && !gProps.searchResult.value?.desc"
      href="javascript:" class="desc"
      @click="onDescClick">[{{ desc![1] + 1 }}]</a>
  </sup>
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
