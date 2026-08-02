<script setup lang="ts">
import { computed, ref } from "vue";
import MarkdownIt from "markdown-it";
import { parseSpec } from "../lib/specParser";

const props = defineProps<{ content: string }>();

const md = new MarkdownIt({ html: false, linkify: true, breaks: true });

function render(text: string): string {
  return md.render(text);
}

const parsed = computed(() => parseSpec(props.content));
const hasSections = computed(
  () =>
    !!parsed.value.purpose ||
    parsed.value.requirements.length > 0 ||
    parsed.value.topScenarios.length > 0 ||
    !!parsed.value.limitations ||
    !!parsed.value.nonGoals
);

const openReqs = ref<Set<number>>(new Set());
const openScen = ref<Set<string>>(new Set());

function toggleReq(i: number) {
  const next = new Set(openReqs.value);
  next.has(i) ? next.delete(i) : next.add(i);
  openReqs.value = next;
}

function toggleScen(key: string) {
  const next = new Set(openScen.value);
  next.has(key) ? next.delete(key) : next.add(key);
  openScen.value = next;
}
</script>

<template>
  <div class="space-y-6">
    <section v-if="parsed.purpose" id="o-purpose" class="scroll-mt-24">
      <h3 class="text-xs font-mono uppercase tracking-wider text-emerald-400 mb-2">🎯 Purpose</h3>
      <div class="text-sm text-slate-200 leading-relaxed prose prose-invert max-w-none" v-html="render(parsed.purpose)" />
    </section>

    <section v-if="parsed.requirements.length" id="o-reqs" class="scroll-mt-24">
      <h3 class="text-xs font-mono uppercase tracking-wider text-emerald-400 mb-2">
        📌 Requirements
      </h3>
      <div class="space-y-2">
        <div
          v-for="(req, i) in parsed.requirements"
          :key="i"
          class="rounded-lg border border-slate-800 bg-slate-900/60 overflow-hidden"
        >
          <button
            @click="toggleReq(i)"
            class="w-full flex items-center gap-2 px-3 py-2.5 text-left hover:bg-slate-800/60 transition"
          >
            <span
              :class="openReqs.has(i) ? 'rotate-90' : ''"
              class="text-slate-500 text-xs transition-transform shrink-0"
            >▶</span>
            <span class="text-sm font-semibold text-slate-100 flex-1">{{ req.name }}</span>
            <span
              v-if="req.scenarios.length"
              class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-800 text-cyan-300"
            >{{ req.scenarios.length }} scenarios</span>
          </button>
          <div
            class="grid transition-[grid-template-rows] duration-300"
            :class="openReqs.has(i) ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
          >
            <div class="overflow-hidden">
              <div class="px-3 pb-3 pt-1 space-y-2">
                <div
                  v-if="req.description"
                  class="text-sm text-slate-300 prose prose-invert max-w-none"
                  v-html="render(req.description)"
                />
                <div v-if="req.scenarios.length" class="space-y-1.5 border-l border-slate-700 pl-3">
                  <div
                    v-for="(sc, j) in req.scenarios"
                    :key="j"
                    class="rounded border border-slate-800/80 bg-slate-950/40 overflow-hidden"
                  >
                    <button
                      @click="toggleScen(i + ':' + j)"
                      class="w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-slate-800/40 transition"
                    >
                      <span
                        :class="openScen.has(i + ':' + j) ? 'rotate-90' : ''"
                        class="text-slate-600 text-[10px] transition-transform shrink-0"
                      >▶</span>
                      <span class="text-xs font-mono text-cyan-300">{{ sc.name }}</span>
                    </button>
                    <div
                      class="grid transition-[grid-template-rows] duration-300"
                      :class="openScen.has(i + ':' + j) ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
                    >
                      <div class="overflow-hidden">
                        <div class="px-3 pb-3 text-xs text-slate-400 prose prose-invert max-w-none" v-html="render(sc.body)" />
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section v-if="parsed.topScenarios.length" id="o-scen" class="scroll-mt-24">
      <h3 class="text-xs font-mono uppercase tracking-wider text-cyan-400 mb-2">
        🎬 Scenarios
      </h3>
      <div class="space-y-2">
        <div
          v-for="(sc, i) in parsed.topScenarios"
          :key="i"
          class="rounded-lg border border-slate-800 bg-slate-900/60 overflow-hidden"
        >
          <button
            @click="toggleScen('top:' + i)"
            class="w-full flex items-center gap-2 px-3 py-2.5 text-left hover:bg-slate-800/60 transition"
          >
            <span
              :class="openScen.has('top:' + i) ? 'rotate-90' : ''"
              class="text-slate-500 text-xs transition-transform shrink-0"
            >▶</span>
            <span class="text-sm font-mono text-cyan-300">{{ sc.name }}</span>
          </button>
          <div
            class="grid transition-[grid-template-rows] duration-300"
            :class="openScen.has('top:' + i) ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
          >
            <div class="overflow-hidden">
              <div class="px-3 pb-3 text-sm text-slate-300 prose prose-invert max-w-none" v-html="render(sc.body)" />
            </div>
          </div>
        </div>
      </div>
    </section>

    <section v-if="parsed.limitations" id="o-lim" class="scroll-mt-24">
      <h3 class="text-xs font-mono uppercase tracking-wider text-amber-400 mb-2">
        ⚠️ Known Limitations
      </h3>
      <div class="text-sm text-slate-200 prose prose-invert max-w-none" v-html="render(parsed.limitations)" />
    </section>

    <section v-if="parsed.nonGoals" class="scroll-mt-24">
      <h3 class="text-xs font-mono uppercase tracking-wider text-slate-400 mb-2">
        🚫 Non-Goals
      </h3>
      <div class="text-sm text-slate-300 prose prose-invert max-w-none" v-html="render(parsed.nonGoals)" />
    </section>

    <p v-if="!hasSections" class="text-xs text-slate-500">
      This file has no structured sections (Purpose / Requirements / Scenarios / Known Limitations).
    </p>
  </div>
</template>
