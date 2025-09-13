import type { Ref } from 'vue';

export interface GlobalProps {
  options: {
    hideSuccessors: boolean,
    hidePredecessors: boolean,
    searchText: string,
  },
  items: Map<number, Item[]>,
  predecessors: Map<number, Link[]>,
  searchResult: Ref<Set<Item> | undefined>,
  resolveLink: (code: number, time: number, rev: boolean) => Item,
}

export interface CodesJson {
  items: Item[],
  descriptions: Record<number, string[]>,
}

export interface Item {
  code: number,
  name: string,
  start: number,
  end?: number,
  successors?: Link[],
  children?: Item[],

  parent?: Item,
  action?: Action,
  act?: () => void,
  isSearchHit?: boolean,
}

export enum Action {
  // Open only.
  Open,
  // Close only.
  Close,
  // Focus only.
  Focus,
  // Open if the item is a folder, focus and scroll.
  OpenFocusScroll,
  // Close and focus.
  CloseFocus,
}

export interface Link {
  time?: number,
  code: number,
  desc_id?: number,
  desc?: string,
}

export interface LinkZip {
  codes: {
    code: number,
    desc?: string,
  }[],
  time: number,
  rev: boolean,
}

export function timeOrDefault(link: Link, item: Item): number {
  return link.time != undefined ? link.time : item.end!;
}

export function scrollToItem(item: Item) {
  item.action = Action.OpenFocusScroll;
  while (item.act == undefined) {
    if (item.parent == undefined) return;
    item = item.parent;
    item.action = Action.Open;
  }
  item.act();
}
