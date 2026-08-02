<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import TaskList from "./components/TaskList.vue";
import OpenQuestions from "./components/OpenQuestions.vue";
import SpecDetail from "./components/SpecDetail.vue";
import SpecOutline from "./components/SpecOutline.vue";
import { parseSpec } from "./lib/specParser";

const rootPath = ref("");
const specs = ref<Array<{ path: string; relative: string; section: string }>>([]);
const changes = ref<Array<{ path: string; relative: string; section: string }>>([]);
const activeFilePath = ref("");
const activeFileContent = ref("");
const activeSection = ref("");
const viewMode = ref<"outline" | "doc">("outline");
const openQuestionsText = ref("");
const unlisten = ref<(() => void) | null>(null);
const errorMsg = ref("");
const loading = ref(false);
const pathInput = ref("");

const activeTitle = computed(() => {
  const parts = activeFilePath.value.split("/");
  const file = parts[parts.length - 1] || "";
  if (file === "spec.md" && parts.length >= 2) {
    const parent = parts[parts.length - 2];
    return parent.charAt(0).toUpperCase() + parent.slice(1);
  }
  return file;
});

const outlineParsed = computed(() =>
  activeSection.value === "specs" && viewMode.value === "outline"
    ? parseSpec(activeFileContent.value)
    : null
);

function goToSection(id: string) {
  document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
}

async function loadFromInput() {
  const p = pathInput.value.trim();
  if (!p) return;
  rootPath.value = p;
  await setRoot(p);
  await loadFolder();
}

function setError(err: unknown) {
  errorMsg.value = typeof err === "string" ? err : err instanceof Error ? err.message : String(err);
  console.error(errorMsg.value, err);
}

async function pickRoot() {
  errorMsg.value = "";
  let picked: string | null = null;
  try {
    picked = (await open({ directory: true })) as string | null;
  } catch (err) {
    setError("The folder dialog failed: " + (err as Error).message);
    return;
  }
  if (!picked) return;
  rootPath.value = picked;
  await setRoot(picked);
  await loadFolder();
}

async function setRoot(root: string) {
  try {
    await invoke("set_root", { root });
  } catch (err) {
    console.warn("File watcher not started (non-critical):", err);
  }
}

async function loadFolder() {
  if (!rootPath.value) return;
  loading.value = true;
  errorMsg.value = "";
  try {
    const entries = await invoke<Array<{ path: string; relative: string; section: string }>>(
      "read_openspec_dir",
      { root: rootPath.value }
    );
    specs.value = entries.filter((e) => e.section === "specs");
    changes.value = entries.filter((e) => e.section === "changes");
    const all = [...specs.value, ...changes.value];
    const active = all.find((e) => e.path === activeFilePath.value);
    if (active) {
      const content = await invoke<string>("read_file_content", { path: active.path });
      activeFileContent.value = content;
      extractQuestions(content);
    } else if (all.length > 0) {
      selectFile(all[0]);
    } else {
      activeFileContent.value = "";
      openQuestionsText.value = "";
    }
  } catch (err) {
    setError("Error loading directory: " + (err as Error).message);
    specs.value = [];
    changes.value = [];
  } finally {
    loading.value = false;
  }
}

async function selectFile(entry: { path: string; section?: string }) {
  activeFilePath.value = entry.path;
  activeSection.value = entry.section ?? "";
  viewMode.value = "outline";
  try {
    const content = await invoke<string>("read_file_content", { path: entry.path });
    activeFileContent.value = content;
    extractQuestions(content);
  } catch (err) {
    setError("Error reading file: " + (err as Error).message);
  }
}

async function updateContent(newContent: string) {
  activeFileContent.value = newContent;
  extractQuestions(newContent);
  try {
    await invoke("save_file_content", { path: activeFilePath.value, content: newContent });
  } catch (err) {
    setError("Error saving file: " + (err as Error).message);
  }
}

function extractQuestions(content: string) {
  const match = content.match(/## Open Questions([\s\S]*?)(?=\n## |$)/);
  openQuestionsText.value = match ? match[1].trim() : "";
}

function saveQuestions(newQuestions: string) {
  let content = activeFileContent.value;
  if (content.includes("## Open Questions")) {
    content = content.replace(
      /## Open Questions[\s\S]*?(?=\n## |$)/,
      `## Open Questions\n${newQuestions}\n\n`
    );
  } else {
    content += `\n\n## Open Questions\n${newQuestions}`;
  }
  updateContent(content);
}

onMounted(async () => {
  unlisten.value = await listen("fs-change", () => loadFolder());
});

onUnmounted(() => {
  if (unlisten.value) unlisten.value();
});
</script>

<template>
  <div class="flex h-screen bg-slate-950 text-slate-100 font-sans">
    <aside class="w-64 border-r border-slate-800 bg-slate-900/50 p-4 flex flex-col shrink-0">
      <h2 class="text-xs font-mono font-semibold uppercase tracking-wider text-slate-400 mb-4">
        OpenSpec Explorer
      </h2>

      <button
        @click="pickRoot"
        class="mb-2 w-full text-xs px-3 py-2 rounded bg-slate-800 hover:bg-slate-700 text-emerald-300 font-mono transition"
      >
        {{ rootPath ? "📂 Change folder" : "📂 Select OpenSpec folder" }}
      </button>
      <div class="flex gap-1 mb-3">
        <input
          v-model="pathInput"
          type="text"
          placeholder="/path/to/repo"
          class="flex-1 min-w-0 text-[10px] font-mono px-2 py-1 rounded bg-slate-900 border border-slate-800 text-slate-300 focus:outline-none focus:border-emerald-600"
        />
        <button
          @click="loadFromInput"
          class="text-[10px] px-2 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono"
        >
          Load
        </button>
      </div>
      <p
        v-if="rootPath"
        class="text-[10px] font-mono text-slate-500 truncate mb-3"
        :title="rootPath"
      >
        {{ rootPath }}
      </p>

      <div class="space-y-4 overflow-y-auto flex-1">
        <div v-if="specs.length">
          <h3 class="text-[10px] font-mono uppercase tracking-wider text-slate-500 mb-1">
            📄 Specs
          </h3>
          <div class="space-y-1">
            <button
              v-for="entry in specs"
              :key="entry.path"
              @click="selectFile(entry)"
              :class="[
                'w-full text-left text-xs p-2 rounded truncate font-mono transition',
                activeFilePath === entry.path
                  ? 'bg-slate-800 text-emerald-400 font-bold'
                  : 'text-slate-400 hover:bg-slate-800/50 hover:text-slate-200',
              ]"
            >
              {{ entry.relative }}
            </button>
          </div>
        </div>

        <div v-if="changes.length">
          <h3 class="text-[10px] font-mono uppercase tracking-wider text-slate-500 mb-1">
            🛠️ Changes
          </h3>
          <div class="space-y-1">
            <button
              v-for="entry in changes"
              :key="entry.path"
              @click="selectFile(entry)"
              :class="[
                'w-full text-left text-xs p-2 rounded truncate font-mono transition',
                activeFilePath === entry.path
                  ? 'bg-slate-800 text-emerald-400 font-bold'
                  : 'text-slate-400 hover:bg-slate-800/50 hover:text-slate-200',
              ]"
            >
              {{ entry.relative }}
            </button>
          </div>
        </div>

        <p
          v-if="!specs.length && !changes.length && rootPath"
          class="text-xs text-slate-400 leading-relaxed"
        >
          No OpenSpec structure found at
          <code class="font-mono text-emerald-400">{{ rootPath }}</code
          >. Expected:
          <code class="font-mono">openspec/specs</code> and
          <code class="font-mono">openspec/changes</code>.
        </p>

        <p v-if="!specs.length && !changes.length && !rootPath" class="text-xs text-slate-500 leading-relaxed">
          Select a folder containing <code class="font-mono">openspec/specs</code> and
          <code class="font-mono">openspec/changes</code>.
        </p>
      </div>
    </aside>

    <main class="flex-1 overflow-y-auto">
      <div
        v-if="errorMsg"
        class="mx-6 mt-6 mb-2 p-3 rounded border border-red-800 bg-red-950/40 text-red-300 text-xs font-mono"
      >
        ⚠️ {{ errorMsg }}
      </div>
      <div v-if="loading" class="mx-6 mt-6 mb-2 text-xs font-mono text-slate-500">Loading…</div>

      <div
        v-if="activeFilePath"
        class="sticky top-0 z-20 -mx-0 px-6 py-3 bg-slate-950/95 backdrop-blur border-b border-slate-800"
      >
        <div class="max-w-4xl">
          <div class="flex items-baseline justify-between gap-4">
            <h1 class="text-lg font-bold text-slate-100 truncate">{{ activeTitle }}</h1>
            <span class="text-[10px] font-mono text-slate-500 truncate" :title="activeFilePath">
              {{ activeFilePath }}
            </span>
          </div>
          <div class="flex items-center gap-2 mt-2 flex-wrap">
            <div class="flex flex-wrap gap-1.5">
              <button
                v-if="outlineParsed?.purpose"
                @click="goToSection('o-purpose')"
                class="text-[11px] px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono"
              >
                🎯 Purpose
              </button>
              <button
                v-if="outlineParsed?.requirements.length"
                @click="goToSection('o-reqs')"
                class="text-[11px] px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono"
              >
                📌 Requirements ({{ outlineParsed.requirements.length }})
              </button>
              <button
                v-if="outlineParsed?.topScenarios.length"
                @click="goToSection('o-scen')"
                class="text-[11px] px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono"
              >
                🎬 Scenarios ({{ outlineParsed.topScenarios.length }})
              </button>
              <button
                v-if="outlineParsed?.limitations"
                @click="goToSection('o-lim')"
                class="text-[11px] px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono"
              >
                ⚠️ Known Limitations
              </button>
              <button
                @click="goToSection('open-questions')"
                class="text-[11px] px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-amber-300 font-mono"
              >
                ❓ Open Questions
              </button>
            </div>
            <div class="flex-1"></div>
            <div
              v-if="activeSection === 'specs'"
              class="flex gap-1 shrink-0 rounded border border-slate-800 p-0.5 bg-slate-900/60"
              title="Change view"
            >
              <button
                @click="viewMode = 'outline'"
                :class="[
                  'p-1.5 rounded transition',
                  viewMode === 'outline'
                    ? 'bg-slate-800 text-emerald-300'
                    : 'text-slate-500 hover:text-slate-300',
                ]"
                title="Outline"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />
                </svg>
              </button>
              <button
                @click="viewMode = 'doc'"
                :class="[
                  'p-1.5 rounded transition',
                  viewMode === 'doc'
                    ? 'bg-slate-800 text-emerald-300'
                    : 'text-slate-500 hover:text-slate-300',
                ]"
                title="Document"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M7 3h7l5 5v13H7z" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M14 3v5h5" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M10 12h5M10 16h5" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="activeFileContent" class="px-6 py-6 max-w-4xl">
        <TaskList :rawContent="activeFileContent" @updateContent="updateContent" />

        <div class="mt-8">
          <SpecOutline
            v-if="activeSection === 'specs' && viewMode === 'outline'"
            :content="activeFileContent"
          />
          <SpecDetail v-else :content="activeFileContent" />
        </div>

        <div id="open-questions" class="mt-6 scroll-mt-28">
          <OpenQuestions :questionsText="openQuestionsText" @saveQuestions="saveQuestions" />
        </div>
      </div>
    </main>
  </div>
</template>
