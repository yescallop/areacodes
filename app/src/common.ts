import type { ComputedRef, Ref } from 'vue';

export interface GlobalProps {
    predecessors: ComputedRef<Map<number, Link[]>>,
    reversed: Ref<boolean>,
}

export interface Item {
    code: number,
    name: string,
    start: number,
    end?: number,
    successors?: Link[],
    children?: Item[],
}

export interface Link {
    time?: number,
    code: number,
}

export function time_or_default(link: Link, item: Item): number {
    return link.time != undefined ? link.time : item.end!;
}
