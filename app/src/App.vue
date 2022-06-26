<script setup lang="ts">
import { ref } from 'vue';
import TreeItem from './TreeItem.vue';
import codesUrl from '../../codes.json?url';

const codes = ref<any[]>([{
  code: 233333,
  name: "加载中...",
  start: new Date().getFullYear(),
}]);
codes.value[0].children = codes.value;

fetch(codesUrl)
  .then(resp => resp.json())
  .then(json => codes.value = json);

const guide = {
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
      name: "绿色行表示上方代码的叶区域在指定年份被部分或全部划为指定代码的叶区域",
      start: 1980,
      sus: [{ code: 1, time: 2000 }]
    },
    {
      code: 5,
      name: "棕色行可以说是绿色行的弱化，具体是什么意思等我再想想（",
      start: 1980,
      sus: [{ code: 2, time: 2010, opt: true }]
    }
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
</style>