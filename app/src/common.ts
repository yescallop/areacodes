export interface GlobalProps {
  options: {
    hideSuccessors: boolean,
    hidePredecessors: boolean,
  },
  items: Map<number, Item[]>,
  predecessors: Map<number, Link[]>,
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
}

export enum Action {
  // Open only.
  Open,
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
}

export interface LinkZipped {
  codes: number[],
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
