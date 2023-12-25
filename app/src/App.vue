<script setup lang="ts">
import { computed, provide, reactive, ref, watch } from 'vue';
import type { CodesJson, GlobalProps, Item, Link } from './common';
import { timeOrDefault, scrollToItem, Action } from './common';
import TreeItem from './components/TreeItem.vue';
import codesUrl from '../../codes.json?url';

const itemArr = ref<Item[]>([{
  code: 233333,
  name: "加载中...",
  start: new Date().getFullYear(),
}]);

const guide: Item = {
  code: 0,
  name: "凡例",
  start: 1980,
  action: localStorage.getItem("closeGuide") == null ? Action.Open : undefined,
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
      action: Action.Open,
      children: [
        {
          code: 4,
          name: "向右的箭头表明代码的后继",
          start: 1980,
          successors: [{ code: 5, time: 1990, details: "这是一条变更详情" }]
        },
        {
          code: 5,
          name: "向左的箭头表明代码的前身",
          start: 1990,
        },
      ]
    },
    {
      code: 6,
      name: "支持键盘操作（Tab、回车、退格）",
      start: 1980,
    },
    {
      code: 7,
      name: "仅支持代码、名称前缀搜索",
      start: 1980,
    }
  ]
};

const options = reactive({
  hideSuccessors: false,
  hidePredecessors: false,
  searchText: "",
});

const items = new Map<number, Item[]>();
const predecessors = new Map<number, Link[]>();

let indexMap: Map<string, Item[]> | undefined = new Map<string, Item[]>();
const indexArr: { name: string; items: Item[]; }[] = [];

const searchResult = computed(() => {
  let text = options.searchText;
  if (text.length == 0) {
    return undefined;
  }

  let res = new Set<Item>();
  if (/^[1-9]\d(\d{2}){0,2}$/.test(text)) {
    let code = parseInt(text);
    while (code < 100000) code *= 100;
    items.get(code)?.forEach(item => {
      searchHit(item);
      res.add(item);
    });
  } else {
    let i = binarySearch(indexArr, t => t.name.localeCompare(text));
    while (i < indexArr.length) {
      let { name, items } = indexArr[i];
      if (!name.startsWith(text)) break;
      items.forEach(item => {
        searchHit(item);
        res.add(item);
      });
      i += 1;
    }
  }

  for (const item of res) {
    if (item.parent != undefined) res.add(item.parent);
    if (item.isSearchHit) {
      item.children?.forEach(child => {
        if (res.has(child) && !child.isSearchHit) {
          // Move it to the end of the set in order to ensure
          // that its successors and predecessors are added.
          res.delete(child);
        }
        child.isSearchHit = true;
        res.add(child);
      });

      if (!options.hideSuccessors)
        addSuccessors(item, item.start, res);
      if (!options.hidePredecessors)
        addPredecessors(item, Infinity, res);
      item.isSearchHit = undefined;
    }
  }
  return res;
});

function searchHit(item: Item) {
  item.isSearchHit = true;
  item.action = Action.Open;
  while (item.parent != undefined) {
    item.parent.action = Action.Open;
    item = item.parent;
  }
}

function resolveLink(code: number, time: number, rev: boolean): Item {
  if (rev) time -= 1;
  // The items are by default descending in time.
  return items.get(code)!.find(item => time >= item.start)!;
}

function addSuccessors(item: Item, after: number, res: Set<Item>) {
  item.successors?.forEach(link => {
    let time = timeOrDefault(link, item);
    if (time > after) {
      let su = resolveLink(link.code, time, false);
      res.add(su);
      addSuccessors(su, time, res);
    }
  });
}

function addPredecessors(item: Item, before: number, res: Set<Item>) {
  predecessors.get(item.code)?.forEach(link => {
    let time = link.time!;
    if (time >= item.start && time < before && (item.end == undefined || time < item.end)) {
      let pre = resolveLink(link.code, time, true);
      res.add(pre);
      addPredecessors(pre, time, res);
    }
  });
}

watch(searchResult, res => {
  if (res == undefined) {
    itemArr.value.forEach(item => {
      item.action = Action.Close;
      if (item.act != undefined) item.act();
    });
  } else {
    res.forEach(item => {
      if (item.act != undefined) item.act();
    });
  }
});

const props: GlobalProps = { options, items, predecessors, searchResult, resolveLink };
provide('props', props);

insertItem(guide, []);

fetch(codesUrl)
  .then(resp => resp.json())
  .then((resp: CodesJson) => {
    resp.items.forEach(item => insertItem(item, resp.details));
    createIndexArr();
    scrollToHash();
    itemArr.value = resp.items;
  });

function insertItem(item: Item, details: string[], parent?: Item) {
  let arr = items.get(item.code);
  if (arr == undefined) {
    arr = [];
    items.set(item.code, arr);
  }
  arr.push(item);

  arr = indexMap!.get(item.name);
  if (arr == undefined) {
    arr = [];
    indexMap!.set(item.name, arr);
  }
  arr.push(item);

  item.successors?.forEach(link => {
    if (link.id != undefined) {
      link.details = details[link.id];
      link.id = undefined;
    }
    let links = predecessors.get(link.code);
    if (links == undefined) {
      links = [];
      predecessors.set(link.code, links);
    }
    links.push({ time: timeOrDefault(link, item), code: item.code, details: link.details });
  });
  item.children?.forEach(child => insertItem(child, details, item));
  item.parent = parent;
}

function createIndexArr() {
  indexMap!.forEach((items, name) => indexArr.push({ name, items }));
  indexMap = undefined;
  indexArr.sort((a, b) => a.name.localeCompare(b.name));
}

// Stolen from `slice::binary_search_by` in Rust.
// In both cases a nonnegative index is returned.
function binarySearch<T>(arr: T[], f: (t: T) => number): number {
  let size = arr.length;
  let left = 0;
  let right = size;
  while (left < right) {
    let mid = left + (size >>> 1);
    let cmp = f(arr[mid]);
    if (cmp < 0) {
      left = mid + 1;
    } else if (cmp > 0) {
      right = mid;
    } else {
      return mid;
    }
    size = right - left;
  }
  return left;
}

window.onhashchange = scrollToHash;

function scrollToHash() {
  options.searchText = "";
  let item = locateHash();
  if (item != undefined) scrollToItem(item);
}

function locateHash(): Item | undefined {
  if (!location.hash.length) return;
  let id = location.hash.substring(1);
  let parts = id.split(':');
  if (parts.length == 2) {
    let code = parseInt(parts[0]);
    let time = parseInt(parts[1]);
    let item = props.items.get(code)?.find(item => time == item.start);
    if (item != undefined) return item;
    window.alert("该代码不存在！");
  }
}
</script>

<template>
  <header>
    <h1><a href="">行政区划代码数据集</a></h1>
    <a title="GitHub" href="https://github.com/yescallop/areacodes">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 16 16">
        <path fill-rule="evenodd"
          d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
      </svg>
    </a>
    <fieldset id="options">
      <legend>选项</legend>
      <label><input type="checkbox" v-model="options.hideSuccessors" />隐藏后继</label>
      <label><input type="checkbox" v-model="options.hidePredecessors" />隐藏前身</label>
    </fieldset>
    <ul id="guide" class="top">
      <TreeItem :item="guide" />
    </ul>
  </header>
  <main>
    <label>搜索：<input type="search" v-model="options.searchText" /></label>
    <ul class="top">
      <TreeItem v-for="it in itemArr" :item="it" />
    </ul>
  </main>
</template>

<style>
body {
  font-family: 'Roboto Mono', 'Noto Sans SC';
}

h1 {
  display: inline-block;
  margin-left: 1ch;
  margin-right: .5ch;
  font-size: x-large;
}

a {
  color: inherit;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

ul {
  list-style-type: none;
  padding-left: 2ch;
}

.top {
  padding-left: 0;
}

#guide rt {
  display: none;
}

#options {
  width: fit-content;
}

#options label:not(:last-child) {
  margin-right: 1ch;
}
</style>