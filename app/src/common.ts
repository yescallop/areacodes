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
    // flags: parent = 0, terminal = 1
    selected?: number,
    onSelected?: () => void,
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
    item.selected = 1;
    while (item.onSelected == undefined) {
        if (item.parent == undefined) return;
        item = item.parent;
        item.selected = 0;
    }
    item.onSelected();
}
