<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useProjectsStore } from "../stores/projects";

const store = useProjectsStore();
const emit = defineEmits<{
  (e: "openProject", path: string): void;
  (e: "pickFolder"): void;
}>();

const search = ref("");
const pathInput = ref("");
const picking = ref(false);
const newOpen = ref(false);
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

const topPad = computed(() => `calc((100vh - 280px) / 2)`);

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return store.projects;
  return store.projects.filter(
    (p) =>
      p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q)
  );
});

onMounted(async () => {
  if (!store.loaded) await store.load();
});

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer);
});

function onSearchInput() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    search.value = search.value; // noop — keep reactive value
  }, 1000);
}

async function pick() {
  if (picking.value) return;
  picking.value = true;
  try {
    let picked: string | null = null;
    try {
      picked = (await open({ directory: true })) as string | null;
    } catch {
      picked = null;
    }
    if (picked) emit("pickFolder");
  } finally {
    picking.value = false;
  }
}

function loadFromInput() {
  const p = pathInput.value.trim();
  if (p) {
    emit("pickFolder");
    emit("openProject", p);
  }
}
</script>

<template>
  <div class="flex h-screen flex-col bg-slate-950 text-slate-100 font-sans p-6 overflow-y-auto">
    <div class="flex-none" :style="{ height: topPad }"></div>
    <div class="w-full max-w-xl mx-auto">
      <div class="text-center mb-8">
        <div class="text-4xl mb-3">📂</div>
        <h1 class="text-2xl font-bold text-slate-100">OpenSpec Visualizer</h1>
        <p class="text-sm text-slate-400 mt-1">
          {{
            store.projects.length
              ? "Pick a project or open a new one."
              : "Open an OpenSpec folder to get started."
          }}
        </p>
      </div>

      <div v-if="store.projects.length" class="mb-6">
        <div class="flex gap-2">
          <input
            v-model="search"
            type="text"
            placeholder="🔍 Search projects…"
            class="flex-1 min-w-0 text-sm font-mono px-3 py-2 rounded bg-slate-900 border border-slate-700 text-slate-200 focus:outline-none focus:border-emerald-600"
            @input="onSearchInput"
          />
          <button
            @click="newOpen = !newOpen"
            class="shrink-0 text-sm font-mono px-3 py-2 rounded bg-slate-800 hover:bg-slate-700 text-emerald-300 transition cursor-pointer"
            :title="newOpen ? 'Close new project form' : 'Open a new project'"
          >
            <span v-if="!newOpen">＋ New project</span>
            <span v-else>✕</span>
          </button>
        </div>

        <div
          v-if="newOpen"
          class="mt-3 rounded-lg border border-dashed border-slate-700 bg-slate-900/30 p-4"
        >
          <div class="flex gap-2">
            <button
              @click="pick"
              :disabled="picking"
              class="flex-1 text-sm font-mono px-4 py-2.5 rounded bg-emerald-700 hover:bg-emerald-600 text-white transition disabled:opacity-50 cursor-pointer"
            >
              {{ picking ? "Opening…" : "📁 Open a project folder" }}
            </button>
          </div>
          <div class="flex gap-2 mt-3">
            <input
              v-model="pathInput"
              type="text"
              placeholder="/path/to/repo"
              class="flex-1 min-w-0 text-[11px] font-mono px-3 py-1.5 rounded bg-slate-950 border border-slate-700 text-slate-300 focus:outline-none focus:border-emerald-600"
              @keydown.enter="loadFromInput"
            />
            <button
              @click="loadFromInput"
              class="text-[11px] font-mono px-3 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 transition cursor-pointer"
            >
              Load
            </button>
          </div>
        </div>
      </div>

      <div
        v-if="store.projects.length"
        class="bg-slate-900/50 border border-slate-800 rounded-lg overflow-hidden mb-6"
      >
        <div v-if="filtered.length" class="divide-y divide-slate-800">
          <button
            v-for="p in filtered"
            :key="p.path"
            @click="emit('openProject', p.path)"
            class="w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-slate-800/60 transition cursor-pointer"
          >
            <span class="text-xl leading-none">{{ p.icon }}</span>
            <span class="flex-1 min-w-0">
              <span class="block text-sm font-mono text-slate-200 truncate">{{ p.name }}</span>
              <span class="block text-[10px] font-mono text-slate-500 truncate">{{ p.path }}</span>
            </span>
            <svg
              class="w-4 h-4 text-slate-600 shrink-0"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
            >
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        </div>
        <p v-else class="px-4 py-4 text-xs font-mono text-slate-500">
          No projects match “{{ search }}”.
        </p>
      </div>

      <div
        v-if="!store.projects.length"
        class="rounded-lg border border-dashed border-slate-700 bg-slate-900/30 p-4"
      >
        <div class="flex gap-2">
          <button
            @click="pick"
            :disabled="picking"
            class="flex-1 text-sm font-mono px-4 py-2.5 rounded bg-emerald-700 hover:bg-emerald-600 text-white transition disabled:opacity-50 cursor-pointer"
          >
            {{ picking ? "Opening…" : "📁 Open a project folder" }}
          </button>
        </div>
        <div class="flex gap-2 mt-3">
          <input
            v-model="pathInput"
            type="text"
            placeholder="/path/to/repo"
            class="flex-1 min-w-0 text-[11px] font-mono px-3 py-1.5 rounded bg-slate-950 border border-slate-700 text-slate-300 focus:outline-none focus:border-emerald-600"
            @keydown.enter="loadFromInput"
          />
          <button
            @click="loadFromInput"
            class="text-[11px] font-mono px-3 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 transition cursor-pointer"
          >
            Load
          </button>
        </div>
      </div>
    </div>
    <div class="flex-1 min-h-8"></div>
  </div>
</template>
