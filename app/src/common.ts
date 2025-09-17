import type { Ref } from 'vue';

export interface GlobalProps {
  options: {
    hideSuccessors: boolean,
    hidePredecessors: boolean,
    searchText: string,
  },
  items: Map<number, Item[]>,
  predecessors: Map<number, Link[]>,
  descriptions: Map<number, string[]>,
  searchResult: Ref<SearchResult | undefined>,
  resolve: (code: number, time: number) => Item | undefined,
  pushHistory: (item: Item) => void,
}

export interface SearchResult {
  items: Set<Item>,
  hits: Set<Item>,
  links: Set<number>,
  desc?: [number, number],
}

export interface CodesJson {
  items: Item[],
  descriptions: Record<string, string[]>,
}

export interface Item {
  code: number,
  name: string,
  start: number,
  end?: number,
  succ?: Link[],
  children?: Item[],

  guide?: boolean,
  root?: boolean,
  parent?: Item,
  action?: Action,
  act?: () => void,
}

export enum Action {
  None = 0,
  Open = 1,
  Close = 2,
  Focus = 4,
  Scroll = 8,
}

export interface Link {
  time?: number,
  code: number,
  desc?: number,
}

export interface LinkCode {
  code: number,
  desc?: number,
  showDesc?: boolean,
}

export interface LinkZip {
  codes: LinkCode[],
  time: number,
  rev: boolean,
}

export function timeOrDefault(link: Link, item: Item): number {
  return link.time != undefined ? link.time : item.end!;
}

export function inUse(item: Item, time: number): boolean {
  return time >= item.start && (item.end == undefined || time < item.end);
}

export function inUseRange(item: Item, start: number, end?: number): boolean {
  if (end == undefined) {
    return item.end == undefined || start < item.end;
  } else if (item.end == undefined) {
    return item.start < end;
  } else {
    return end > item.start && start < item.end;
  }
}

export function encodeLink(src: number, dst: number, time: number): number {
  return src * 1e10 + dst * 1e4 + time
}

export function decodeLink(n: number): [number, number, number] {
  const time = n % 1e4;
  n = Math.trunc(n / 1e4);
  const dst = n % 1e6;
  n = Math.trunc(n / 1e6);
  return [n, dst, time];
}

export function exposeAnd(item: Item, action: Action) {
  item.action = action;
  while (item.act == undefined) {
    if (item.parent == undefined) return;
    item = item.parent;
    item.action = Action.Open;
  }
  item.act();
}
