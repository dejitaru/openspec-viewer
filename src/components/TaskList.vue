<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{ rawContent: string }>();
const emit = defineEmits<{ (e: "updateContent", content: string): void }>();

const tasks = computed(() => {
  if (!props.rawContent) return [];
  const lines = props.rawContent.split("\n");
  return lines
    .map((line, index) => {
      const match = line.match(/^\s*-\s*\[([ xX])\]\s*(.+)/);
      if (match) {
        return { index, checked: match[1] !== " ", text: match[2] };
      }
      return null;
    })
    .filter(Boolean) as Array<{ index: number; checked: boolean; text: string }>;
});

const progress = computed(() => {
  if (tasks.value.length === 0) return 0;
  const done = tasks.value.filter((t) => t.checked).length;
  return Math.round((done / tasks.value.length) * 100);
});

function toggleTask(task: { index: number; checked: boolean }) {
  const lines = props.rawContent.split("\n");
  const newStatus = task.checked ? " " : "x";
  lines[task.index] = lines[task.index].replace(/\[[ xX]\]/, `[${newStatus}]`);
  emit("updateContent", lines.join("\n"));
}
</script>

<template>
  <div v-if="tasks.length" class="my-4 p-4 bg-slate-900 rounded-lg border border-slate-800">
    <div class="flex justify-between items-center mb-2">
      <span class="text-xs font-mono uppercase text-slate-400">Task Progress</span>
      <span class="text-xs font-mono text-emerald-400 font-bold">{{ progress }}%</span>
    </div>

    <div class="w-full h-2 bg-slate-800 rounded-full overflow-hidden mb-4">
      <div
        class="h-full bg-emerald-500 transition-all duration-300"
        :style="{ width: progress + '%' }"
      ></div>
    </div>

    <div class="space-y-2">
      <div
        v-for="task in tasks"
        :key="task.index"
        @click="toggleTask(task)"
        class="flex items-center gap-3 cursor-pointer p-2 rounded hover:bg-slate-800/60 transition-colors"
      >
        <input
          type="checkbox"
          :checked="task.checked"
          @click.stop
          @change="toggleTask(task)"
          class="rounded border-slate-700 text-emerald-500 focus:ring-0"
        />
        <span
          :class="{ 'line-through text-slate-500': task.checked, 'text-slate-200': !task.checked }"
          class="text-sm"
        >
          {{ task.text }}
        </span>
      </div>
    </div>
  </div>
</template>
