<script setup lang="ts">
import { ref } from 'vue';
import TreeItem from './TreeItem.vue';
import codesUrl from '../../codes.json?url';

const codes = ref<any[]>([]);
fetch(codesUrl)
  .then(resp => resp.json())
  .then(json => codes.value = json);

const example = {
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
      end: new Date().getFullYear(),
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
    <ul class="top">
      <TreeItem :item="example"></TreeItem>
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

ul {
  list-style-type: none;
  padding-left: 2ch;
}

.top {
  padding-left: 0;
}
</style>