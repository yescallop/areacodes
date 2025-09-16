<script setup lang="ts">
import { computed, provide, reactive, ref, watch } from 'vue';
import type { CodesJson, GlobalProps, Item, Link, SearchResult } from './common';
import { timeOrDefault, Action, inUse, inUseRange, encodeLink, decodeLink } from './common';
import TreeItem from './components/TreeItem.vue';
import codesUrl from '../../codes.json?url';

const root = ref<Item>({
  code: 0,
  name: "中华人民共和国",
  start: 1949,
  root: true,
});

const guide: Item = {
  code: 0,
  name: "凡例",
  start: 0,
  action: localStorage.getItem("closeGuide") == null ? Action.Open : undefined,
  children: [
    {
      code: 1,
      name: "圆括号内为在用时间",
      start: 1980,
    },
    {
      code: 2,
      name: "灰色为弃用代码",
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
      name: "支持键盘操作（加、减、退格）",
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

let nameIndexMap: Map<string, Item[]> | undefined = new Map<string, Item[]>();
const nameIndex: { name: string; items: Item[]; }[] = [];

const timeIndex: Set<number>[][] = [];

const searchResult = computed(() => {
  const text = options.searchText;
  if (text.length == 0) {
    return undefined;
  }

  const res: SearchResult = { items: new Set(), links: new Set() };
  let start: number | undefined;
  let end: number | undefined;

  if (/^(19|20)\d{2}\.(\d+)?$/.test(text)) {
    const parts = text.split('.');
    const time = parseInt(parts[0]!);

    const arr = timeIndex[time];
    if (arr == undefined) return res;

    const processLink = (link: number) => {
      const [src, dst, time] = decodeLink(link);
      for (const item of [resolve(src, time - 1)!, resolve(dst, time)!]) {
        searchHit(item, false);
        res.items.add(item);
      }
      res.links.add(link);
    };

    if (parts[1] == '') {
      arr.forEach(subArr => subArr.forEach(processLink));
    } else {
      const idx = parseInt(parts[1]!);
      if (idx <= 0) return res;
      res.desc = [time, idx - 1];
      arr[idx]?.forEach(processLink);
    }
  } else if (/^\d{6}(,\d{4}(-(\d{4})?)?)?$/.test(text)) {
    let parts = text.split(',');
    const code = parseInt(parts[0]!);

    if (parts.length == 1) {
      items.get(code)?.forEach(item => {
        searchHit(item, true);
        res.items.add(item);
      });
    } else {
      parts = parts[1]!.split('-');
      start = parseInt(parts[0]!);
      if (parts.length == 1) {
        end = start + 1;
      } else if (parts[1] != '') {
        end = parseInt(parts[1]!);
      }

      for (const item of resolveRange(code, start, end)) {
        searchHit(item, true);
        res.items.add(item);
      }
    }
  } else {
    let i = binarySearch(nameIndex, t => t.name.localeCompare(text));
    while (i < nameIndex.length) {
      const { name, items } = nameIndex[i]!;
      if (!name.startsWith(text)) break;
      items.forEach(item => {
        searchHit(item, true);
        res.items.add(item);
      });
      i += 1;
    }
  }

  if (res.items.size == 1) {
    for (const item of res.items)
      item.action = Action.OpenScroll;
  }

  for (const item of res.items) {
    if (item.parent != undefined) res.items.add(item.parent);
    if (item.isSearchHit) {
      item.children?.forEach(child => {
        if (start && !inUseRange(child, start, end))
          return;
        if (res.items.has(child) && !child.isSearchHit) {
          // Move it to the end of the set in order to ensure
          // that its successors and predecessors are added.
          res.items.delete(child);
        }
        child.isSearchHit = true;
        res.items.add(child);
      });

      if (!options.hideSuccessors)
        addSuccessors(item, start ?? 0, res);
      if (!options.hidePredecessors)
        addPredecessors(item, end ?? Infinity, res);
      item.isSearchHit = undefined;
    }
  }
  return res;
});

function searchHit(item: Item, mark: boolean) {
  item.isSearchHit = mark;
  while (item.parent != undefined) {
    item.parent.action = Action.Open;
    item = item.parent;
  }
}

function resolve(code: number, time: number): Item | undefined {
  return items.get(code)?.find(item => inUse(item, time));
}

function resolveRange(code: number, start: number, end?: number): Item[] {
  return items.get(code)?.filter(item => inUseRange(item, start, end)) ?? [];
}

function addSuccessors(item: Item, after: number, res: SearchResult) {
  item.succ?.forEach(link => {
    const time = timeOrDefault(link, item);
    if (time > after) {
      const succ = resolve(link.code, time)!;
      res.items.add(succ);
      res.links.add(encodeLink(item.code, link.code, time));
      addSuccessors(succ, time, res);
    }
  });
}

function addPredecessors(item: Item, before: number, res: SearchResult) {
  predecessors.get(item.code)?.forEach(link => {
    const time = link.time!;
    if (time < before && inUse(item, time)) {
      const pred = resolve(link.code, time - 1)!;
      res.items.add(pred);
      res.links.add(encodeLink(link.code, item.code, time));
      addPredecessors(pred, time, res);
    }
  });
}

watch(searchResult, res => {
  if (res == undefined) {
    root.value.children?.forEach(item => {
      item.action = Action.Close;
      if (item.act != undefined) item.act();
    });
  } else {
    res.items.forEach(item => {
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
  resolve,
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

    createIndex();
    followHash();
    root.value.children = resp.items;
  });

function insertItem(item: Item, parent?: Item) {
  let arr = items.get(item.code);
  if (arr == undefined) {
    arr = [];
    items.set(item.code, arr);
  }
  arr.push(item);

  arr = nameIndexMap!.get(item.name);
  if (arr == undefined) {
    arr = [];
    nameIndexMap!.set(item.name, arr);
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

function createIndex() {
  nameIndexMap!.forEach((items, name) => {
    nameIndex.push({ name, items });

    items.forEach(item => {
      item.succ?.forEach(link => {
        const time = timeOrDefault(link, item);
        const idx = link.desc != undefined ? link.desc + 1 : 0;

        let arr = timeIndex[time];
        if (arr == undefined) {
          arr = [new Set()];
          timeIndex[time] = arr;
        }
        let subArr = arr[idx];
        if (subArr == undefined) {
          subArr = new Set();
          arr[idx] = subArr;
        }
        subArr.add(encodeLink(item.code, link.code, time));
      });
    });
  });
  nameIndexMap = undefined;
  nameIndex.sort((a, b) => a.name.localeCompare(b.name));
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

window.onhashchange = followHash;

function followHash() {
  options.searchText = decodeURIComponent(location.hash.substring(1));
}

import markdownit from 'markdown-it';
const md = markdownit();

// https://github.com/markdown-it/markdown-it/blob/master/docs/architecture.md#renderer
// Remember the old renderer if overridden, or proxy to the default renderer.
const defaultRender = md.renderer.rules.link_open || function (tokens, idx, options, _env, self) {
  return self.renderToken(tokens, idx, options);
};

md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
  // Add a new `target` attribute, or replace the value of the existing one.
  tokens[idx]!.attrSet('target', '_blank');

  // Pass the token to the default renderer.
  return defaultRender(tokens, idx, options, env, self);
};

function render([time, desc]: [number, number]): string {
  const descStr = descriptions.get(time)![desc]!;
  return md.render(descStr.replace(/^#/g, "##"));
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
    <ul id="guide">
      <TreeItem :item="guide" />
    </ul>
  </header>
  <main>
    <div id="search-bar">
      <label>搜索：<input type="search" v-model="options.searchText" /></label>
      <a id="search-link" :href="'#' + encodeURI(options.searchText)">[直链]</a>
      <hr />
    </div>
    <TreeItem id="root" :item="root" />
    <hr v-if="searchResult?.desc" />
    <article v-if="searchResult?.desc" v-html="render(searchResult.desc)"></article>
  </main>
</template>

<style>
html {
  scroll-padding-top: 44px;
}

body {
  font-family: 'Roboto Mono', 'Noto Sans SC';
}

h1 {
  display: inline-block;
  margin: 8px;
  font-size: x-large;
}

h2 {
  font-size: large;
}

h2,
p {
  margin: 8px 0;
}

ol {
  margin: 8px 0;
}

a {
  color: inherit;
  text-decoration: none;
}

article {
  font-family: 'Noto Sans SC';
}

article a {
  color: darkblue;
}

a:hover {
  text-decoration: underline;
}

a:focus:not(:focus-visible) {
  text-decoration: underline;
}

ul {
  list-style-type: none;
  padding-left: 1ch;
}

#guide {
  padding-left: 0;
  margin-top: 8px;
  margin-bottom: 0;
  /* Make the focus ring appear above the search bar */
  position: relative;
  z-index: 1;
}

#guide rt {
  display: none;
}

#root {
  padding-left: 0;
  margin: 8px 0;
}

#root>ul {
  padding-left: 0;
  margin: 0;
}

#search-link {
  margin-left: 1ch;
  color: darkblue;
}

#options {
  width: fit-content;
}

#options label:not(:last-child) {
  margin-right: 1ch;
}

#search-bar {
  padding-top: 8px;
  position: sticky;
  top: 0;
  background: white;
}

hr {
  margin: 8px -8px;
}
</style>
