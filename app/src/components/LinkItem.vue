<script setup lang="ts">
import { inject } from 'vue';
import { type Item, scrollToItem, Action } from '@/common';

defineProps<{
  item: Item;
  enabled: boolean;
  desc?: [number, number];
}>();

const srcItem = inject<Item>('srcItem')!;

function onKeyDown(e: KeyboardEvent) {
  if (e.code == "Backspace") {
    srcItem.action = Action.Focus;
    srcItem.act!();
  }
}

function pushSrc() {
  const hash = `#${srcItem.code}:${srcItem.start}`;
  if (hash != location.hash) {
    history.pushState(null, "", hash);
  }
}
</script>

<template>
  <a :class="{ disabled: !enabled }" :href="enabled ? `#${item.code}:${item.start}` : undefined"
    @click.prevent="if (enabled) { pushSrc(); scrollToItem(item); }" @keydown="onKeyDown">
    <ruby v-if="item.name != srcItem.name">{{ item.code }}<rt>{{
      item.name }}</rt></ruby>
    <template v-else>{{ item.code }}</template>
  </a>
  <a v-if="desc != undefined" :href="`#${desc[0]}.${desc[1]}`" @click="pushSrc" class="desc">[?]</a>
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
