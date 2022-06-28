<script setup lang="ts">
import { computed, provide, ref } from 'vue';
import { type GlobalProps, type Item, type Link, time_or_default } from './common';
import TreeItem from './TreeItem.vue';
import codesUrl from '../../codes.json?url';

const codes = ref<Item[]>([{
  code: 233333,
  name: "加载中...",
  start: new Date().getFullYear(),
}]);
codes.value[0].children = codes.value;

const predecessors = computed(() => {
  let map = new Map<number, Link[]>();
  guide.children?.forEach(item => insert(map, item));
  codes.value.forEach(item => insert(map, item));
  return map;
});
const reversed = ref(false);

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

provide<GlobalProps>('props', { predecessors, reversed });

document.onkeydown = e => {
  if (e.key == "R" || e.key == "r") {
    reversed.value = !reversed.value;
  }
};

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
      name: "定义一条代码的未被子代码（若有）覆盖的区域为其叶区域",
      start: 1980,
    },
    {
      code: 4,
      name: "绿色行表示原代码的叶区域在指定年份被部分或全部划为指定代码的叶区域",
      start: 1980,
      successors: [{ code: 3, time: 2000 }]
    },
  ]
};
</script>

<template>
  <header>
    <h1>行政区划代码数据集</h1>
    <a href="https://github.com/yescallop/areacodes">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 16 16">
        <path fill-rule="evenodd"
          d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
      </svg>
    </a>
    <ul class="top">
      <TreeItem :item="guide"></TreeItem>
    </ul>
    <div id="options">
      <span>选项：</span>
      <input type="checkbox" id="reverse" v-model="reversed" />
      <label for="reverse">反向追溯（R）</label>
    </div>
  </header>
  <main>
    <ul class="top">
      <TreeItem v-for="child in codes" :item="child"></TreeItem>
    </ul>
  </main>
</template>

<style>
@import url('https://fonts.googleapis.com/css2?family=Noto+Sans+SC&family=Roboto+Mono&display=swap');

body {
  font-family: 'Roboto Mono', 'Noto Sans SC';
}

h1 {
  display: inline-block;
  margin-left: 1ch;
  margin-right: .5ch;
  margin-bottom: 0;
  font-size: x-large;
}

ul {
  list-style-type: none;
  padding-left: 2ch;
}

.top {
  padding-left: 0;
}

label {
  padding-right: 1ch;
}
</style>