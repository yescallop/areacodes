<script setup lang="ts">
import type { LinkZipped } from '@/common';
import Link from './Link.vue';

defineProps<{ links: LinkZipped[]; }>();
</script>

<template>
  <ul v-if="links.length" class="links">
    <li v-for="link in links" :class="{ rev: link.rev }">
      <template v-for="(code, index) in link.codes">
        <template v-if="index != 0">,</template>
        <Link :code="code" :time="link.time" :rev="link.rev" />
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
</style>