export interface GlobalProps {
    options: {
        hideSucc: boolean,
        hidePred: boolean,
    },
    items: Map<number, Item[]>,
    predecessors: Map<number, Link[]>,
    locate: (code: number, time: number) => Item | undefined,
    scrollTo: (item: Item) => void,
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
