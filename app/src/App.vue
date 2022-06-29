<script setup lang="ts">
import { computed, provide, reactive, ref } from 'vue';
import { type GlobalProps, type Item, type Link, time_or_default } from './common';
import TreeItem from './TreeItem.vue';
import codesUrl from '../../codes.json?url';

const codes = ref<Item[]>([{
  code: 233333,
  name: "加载中...",
  start: new Date().getFullYear(),
}]);

const predecessors = computed(() => {
  let map = new Map<number, Link[]>();
  guide.children?.forEach(item => insert(map, item));
  codes.value.forEach(item => insert(map, item));
  return map;
});

const options = reactive({
  hide_succ: false,
  hide_pred: false,
});

function insert(map: Map<number, Link[]>, item: Item) {
  item.successors?.forEach(link => {
    let links = map.get(link.code);
    if (links == undefined) {
      links = [];
      map.set(link.code, links);
    }
    links.push({ time: time_or_default(link, item), code: item.code });
  });
  item.children?.forEach(child => insert(map, child));
}

provide<GlobalProps>('props', { predecessors, options });

fetch(codesUrl)
  .then(resp => resp.json())
  .then(json => codes.value = json);

const guide: Item = {
  code: 0,
  name: "凡例",
  start: 1980,
  children: [
    {
      code: 1,
      name: "尖括号内数字为代码的启用年份",
      start: 1980,
    },
    {
      code: 2,
      name: "灰色行表示已弃用的代码",
      start: 1980,
      end: 1990,
    },
    {
      code: 3,
      name: "以箭头起始的行描述新旧代码间的对应关系",
      start: 1980,
      children: [
        {
          code: 4,
          name: "向右的直箭头 (=>) 表明代码的后继",
          start: 1980,
          successors: [{ code: 5, time: 1990 }]
        },
        {
          code: 5,
          name: "向左的直箭头 (<=) 表明代码的前身",
          start: 1990,
        },
      ]
    },
  ]
};
</script>

<template>
  <header>
    <h1>行政区划代码数据集<a href="https://github.com/yescallop/areacodes">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 16 16">
          <path fill-rule="evenodd"
            d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
        </svg>
      </a></h1>
    <fieldset id="options">
      <legend>选项</legend>
      <label><input type="checkbox" id="hide_succ" v-model="options.hide_succ" />隐藏后继</label>
      <label><input type="checkbox" id="hide_pred" v-model="options.hide_pred" />隐藏前身</label>
    </fieldset>
    <ul class="top">
      <TreeItem :item="guide" :open="true"></TreeItem>
    </ul>
  </header>
  <main>
    <ul class="top">
      <TreeItem v-for="child in codes" :item="child" :open="false"></TreeItem>
    </ul>
  </main>
</template>

<style>
@import url('https://fonts.googleapis.com/css2?family=Noto+Sans+SC&family=Roboto+Mono&display=swap');

body {
  font-family: 'Roboto Mono', 'Noto Sans SC';
}

h1 {
  margin-left: 1ch;
  font-size: x-large;
}

h1 a {
  margin-left: .5ch;
}

ul {
  list-style-type: none;
  padding-left: 2ch;
}

.top {
  padding-left: 0;
}

#options {
  width: fit-content;
}

#options label:not(:last-child) {
  margin-right: 1ch;
}
</style>