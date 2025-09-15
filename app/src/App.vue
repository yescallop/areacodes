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
  start: 0,
  action: localStorage.getItem("closeGuide") == null ? Action.Open : undefined,
  children: [
    {
      code: 1,
      name: "黑色为在用代码，后接启用时间",
      start: 1980,
    },
    {
      code: 2,
      name: "灰色为弃用代码，后接在用时间",
      start: 1970,
      end: 1980,
    },
    {
      code: 3,
      name: "向右的绿色箭头表明代码的后继",
      start: 0,
      succ: [{ code: 4, time: 1980, desc: 0 }]
    },
    {
      code: 4,
      name: "向左的红色箭头表明代码的前身",
      start: 0,
    },
    {
      code: 5,
      name: "点击蓝色框内问号查看变更描述",
      start: 0,
    },
    {
      code: 6,
      name: "可键盘操作（Tab、回车、退格）",
      start: 0,
    },
    {
      code: 7,
      name: "支持代码、名称前缀、时间搜索",
      start: 0,
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
const descriptions = new Map<number, string[]>();

descriptions.set(1980, ["这是一条变更描述"]);

let indexMap: Map<string, Item[]> | undefined = new Map<string, Item[]>();
const indexArr: { name: string; items: Item[]; }[] = [];

const searchResult = computed(() => {
  const text = options.searchText;
  if (text.length == 0) {
    return undefined;
  }

  const res = new Set<Item>();
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
      const { name, items } = indexArr[i]!;
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
  item.succ?.forEach(link => {
    const time = timeOrDefault(link, item);
    if (time > after) {
      const su = resolveLink(link.code, time, false);
      res.add(su);
      addSuccessors(su, time, res);
    }
  });
}

function addPredecessors(item: Item, before: number, res: Set<Item>) {
  predecessors.get(item.code)?.forEach(link => {
    const time = link.time!;
    if (time >= item.start && time < before && (item.end == undefined || time < item.end)) {
      const pre = resolveLink(link.code, time, true);
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

    if (itemToScrollTo != undefined) {
      scrollToItem(itemToScrollTo);
      itemToScrollTo = undefined;
    }
  } else {
    res.forEach(item => {
      if (item.act != undefined) item.act();
    });
  }
});

const props: GlobalProps = {
  options,
  items,
  predecessors,
  descriptions,
  searchResult,
  resolveLink
};
provide('props', props);

insertItem(guide);

fetch(codesUrl)
  .then(resp => resp.json())
  .then((resp: CodesJson) => {
    resp.items.forEach(item => insertItem(item));
    for (const [time, arr] of Object.entries(resp.descriptions)) {
      descriptions.set(parseInt(time), arr);
    }

    createIndexArr();
    scrollToHash();
    itemArr.value = resp.items;
  });

function insertItem(item: Item, parent?: Item) {
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

  item.succ?.forEach(link => {
    let links = predecessors.get(link.code);
    if (links == undefined) {
      links = [];
      predecessors.set(link.code, links);
    }
    links.push({ time: timeOrDefault(link, item), code: item.code, desc: link.desc });
  });
  item.children?.forEach(child => insertItem(child, item));
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
    const mid = left + (size >>> 1);
    const cmp = f(arr[mid]!);
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

let itemToScrollTo: Item | undefined = undefined;

function scrollToHash() {
  const item = locateHash();
  if (item === null) {
    return;
  } else if (item == undefined) {
    options.searchText = decodeURIComponent(location.hash.substring(1));
    return;
  }

  if (options.searchText == "") {
    scrollToItem(item);
  } else {
    itemToScrollTo = item;
    options.searchText = "";
  }
}

function locateHash(): Item | null | undefined {
  if (!location.hash.length) return;
  const id = location.hash.substring(1);
  const parts = id.split(':');
  if (parts.length == 2) {
    const code = parseInt(parts[0]!);
    const time = parseInt(parts[1]!);
    const item = props.items.get(code)?.find(item => time == item.start);
    if (item != undefined) return item;
    window.alert("该代码不存在！");
    return null;
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
    <a id="search-link" :href="'#' + encodeURIComponent(options.searchText)">[直链]</a>
    <ul class="top">
      <TreeItem v-for="it in itemArr" :item="it" :key="it.code * 10000 + it.start" />
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
  padding-left: 1ch;
}

.top {
  padding-left: 0;
}

#guide rt {
  display: none;
}

#search-link {
  padding-left: 1ch;
  color: darkblue;
}

#options {
  width: fit-content;
}

#options label:not(:last-child) {
  margin-right: 1ch;
}
</style>
