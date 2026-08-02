<script setup lang="ts">
import { computed } from "vue";
import MarkdownIt from "markdown-it";

const props = defineProps<{ content: string }>();

const md = new MarkdownIt({ html: false, linkify: true, breaks: true });

const body = computed(() => {
  let text = props.content;
  text = text.replace(/## Open Questions[\s\S]*?(?=\n## |$)/, "");
  text = text
    .split("\n")
    .filter((line) => !/^\s*-\s*\[[ xX]\]/.test(line))
    .join("\n");
  return text;
});

const html = computed(() => md.render(body.value));
</script>

<template>
  <article class="prose prose-invert max-w-none" v-html="html" />
</template>
