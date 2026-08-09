<script setup lang="ts">
import type { ParsedSpec } from "../lib/specParser";

defineProps<{
  capabilities: Array<{ capability: string; parsed: ParsedSpec }>;
  selected: { capability: string; index: number } | null;
}>();

const emit = defineEmits<{
  (e: "select", capability: string, index: number): void;
}>();
</script>

<template>
  <aside class="w-64 border-l border-slate-800 bg-slate-900/50 p-4 flex flex-col shrink-0">
    <h3 class="text-[10px] font-mono uppercase tracking-wider text-slate-500 mb-3">
      📌 Requirements
    </h3>
    <div class="space-y-4 overflow-y-auto flex-1">
      <p v-if="!capabilities.some((c) => c.parsed.requirements.length)" class="text-xs text-slate-500">
        No requirements found.
      </p>
      <div v-for="cap in capabilities" :key="cap.capability">
        <h4
          v-if="capabilities.length > 1"
          class="text-[10px] font-mono uppercase tracking-wider text-slate-600 mb-1"
        >
          {{ cap.capability }}
        </h4>
        <div class="space-y-1">
          <button
            v-for="(req, i) in cap.parsed.requirements"
            :key="i"
            @click="emit('select', cap.capability, i)"
            :class="[
              'w-full text-left text-xs p-2 rounded truncate font-mono transition',
              selected?.capability === cap.capability && selected?.index === i
                ? 'bg-slate-800 text-emerald-400 font-bold'
                : 'text-slate-400 hover:bg-slate-800/50 hover:text-slate-200',
            ]"
          >
            {{ req.name }}
          </button>
        </div>
      </div>
    </div>
  </aside>
</template>
