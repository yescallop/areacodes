<script setup lang="ts">
import type { LinkZipped } from '@/common';

const props = defineProps<{ links: LinkZipped[]; }>();
</script>

<template>
  <ul v-if="props.links.length" class="links">
    <li v-for="link in props.links" :class="{ rev: link.rev }">
      <template v-for="code in link.codes">
        <a :href="`#${code}:${link.time - (link.rev ? 1 : 0)}`">{{ code }}</a>
        <span>,</span>
      </template>
      &lt;{{ link.time }}&gt;
    </li>
  </ul>
</template>

<style>
.links {
  color: green;
  padding-left: 5ch;
}

.links li.rev {
  color: darkred;
}

.links li::before {
  content: "=>";
  padding-right: 1ch;
}

.links li.rev::before {
  content: "<=";
}

.links span:last-child {
  display: none;
}
</style>