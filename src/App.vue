<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import MarkdownIt from "markdown-it";
import TaskList from "./components/TaskList.vue";
import OpenQuestions from "./components/OpenQuestions.vue";
import SpecDetail from "./components/SpecDetail.vue";
import ProjectSwitcher from "./components/ProjectSwitcher.vue";
import ProjectHome from "./components/ProjectHome.vue";
import { useProjectsStore } from "./stores/projects";
import { parseSpec, type ParsedSpec } from "./lib/specParser";
import ChangeRequirementsSidebar from "./components/ChangeRequirementsSidebar.vue";
import SettingsModal from "./components/SettingsModal.vue";

const md = new MarkdownIt({ html: false, linkify: true, breaks: true });
function renderMd(text: string): string {
  return md.render(text);
}

const projectsStore = useProjectsStore();
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
const collapsedSpecs = ref(false);
const collapsedChanges = ref(false);
const activeChangeFolder = ref("");
const fileFilter = ref("");
const fileFilterInput = ref<HTMLInputElement | null>(null);
const settingsOpen = ref(false);
const activeTitle = computed(() => {
  if (changeSpecsMode.value) return activeChangeFolder.value;
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

const filteredSpecs = computed(() => filterEntries(specs.value));
const filteredChanges = computed(() => filterEntries(changes.value));

function filterEntries(entries: Array<{ path: string; relative: string; section: string }>) {
  const q = fileFilter.value.trim().toLowerCase();
  if (!q) return entries;
  return entries.filter(
    (e) =>
      e.relative.toLowerCase().includes(q) || e.path.toLowerCase().includes(q)
  );
}

function changeFolderOf(relative: string) {
  const idx = relative.indexOf("/");
  return idx === -1 ? relative : relative.slice(0, idx);
}

type ChangeFileKind = "design" | "proposal" | "tasks" | "specs" | "other";

interface ChangeFileTab {
  kind: ChangeFileKind;
  order: number;
  label: string;
  icon: string;
  path: string;
  relative: string;
}

function specCapabilityOf(folder: string, relative: string): string | null {
  const rel = relative === folder ? relative : relative.slice(folder.length + 1);
  const match = rel.match(/^specs\/(.+?)\/spec\.md$/);
  return match ? match[1] : null;
}

const allGroupedChanges = computed(() => {
  const groups = new Map<string, Array<{ path: string; relative: string; section: string }>>();
  for (const entry of changes.value) {
    const folder = changeFolderOf(entry.relative);
    if (!groups.has(folder)) groups.set(folder, []);
    groups.get(folder)!.push(entry);
  }
  return Array.from(groups.entries()).map(([folder, entries]) => {
    const specFiles: Array<{ path: string; relative: string; capability: string }> = [];
    const tabs: ChangeFileTab[] = [];
    for (const e of entries) {
      const capability = specCapabilityOf(folder, e.relative);
      if (capability) {
        specFiles.push({ path: e.path, relative: e.relative, capability });
        continue;
      }
      const rel = e.relative === folder ? e.relative : e.relative.slice(folder.length + 1);
      if (rel === "design.md") tabs.push({ kind: "design", order: 0, label: "Design", icon: "📐", path: e.path, relative: e.relative });
      else if (rel === "proposal.md") tabs.push({ kind: "proposal", order: 1, label: "Proposal", icon: "📋", path: e.path, relative: e.relative });
      else if (rel === "tasks.md") tabs.push({ kind: "tasks", order: 2, label: "Tasks", icon: "✅", path: e.path, relative: e.relative });
      else tabs.push({ kind: "other", order: 4, label: rel.replace(/\.md$/, ""), icon: "📝", path: e.path, relative: e.relative });
    }
    if (specFiles.length) {
      tabs.push({ kind: "specs", order: 3, label: "Specs", icon: "📄", path: specFiles[0].path, relative: specFiles[0].relative });
    }
    tabs.sort((a, b) => a.order - b.order || a.label.localeCompare(b.label));
    return { folder, entries, tabs, specFiles };
  });
});

const groupedChangeFolders = computed(() => {
  const matching = new Set(filteredChanges.value.map((e) => changeFolderOf(e.relative)));
  return allGroupedChanges.value.filter((g) => matching.has(g.folder));
});

const activeChangeGroup = computed(
  () => allGroupedChanges.value.find((g) => g.folder === activeChangeFolder.value) || null
);

interface SpecBrowserFile {
  capability: string;
  path: string;
  content: string;
  parsed: ParsedSpec;
}

type SpecSection = "purpose" | "requirements" | "scenarios" | "limitations" | "nonGoals" | "openQuestions";

const changeSpecsMode = ref(false);
const changeSpecsFiles = ref<SpecBrowserFile[]>([]);
const specRequirementsMode = ref(false);
const specViewSection = ref<SpecSection>("purpose");
const selectedRequirement = ref<{ capability: string; index: number } | null>(null);

const requirementsMode = computed(
  () =>
    changeSpecsMode.value ||
    (specRequirementsMode.value && activeSection.value === "specs" && viewMode.value === "outline")
);

function showSpecSection(section: SpecSection) {
  specViewSection.value = section;
  specRequirementsMode.value = section === "requirements";
}

function defaultSpecSection(parsed: ParsedSpec | null): SpecSection {
  if (parsed?.purpose) return "purpose";
  if (parsed?.requirements.length) return "requirements";
  if (parsed?.topScenarios.length) return "scenarios";
  if (parsed?.limitations) return "limitations";
  if (parsed?.nonGoals) return "nonGoals";
  return "openQuestions";
}

function onOpenQuestionsClick() {
  if (activeSection.value === "specs" && viewMode.value === "outline") {
    showSpecSection("openQuestions");
  } else {
    goToSection("open-questions");
  }
}

const specBrowserCapabilities = computed<SpecBrowserFile[]>(() => {
  if (changeSpecsMode.value) return changeSpecsFiles.value;
  if (requirementsMode.value && activeFileContent.value) {
    return [
      {
        capability: activeTitle.value,
        path: activeFilePath.value,
        content: activeFileContent.value,
        parsed: parseSpec(activeFileContent.value),
      },
    ];
  }
  return [];
});

const specBrowserOverview = computed<ParsedSpec | null>(() => {
  if (selectedRequirement.value) {
    const cap = specBrowserCapabilities.value.find((c) => c.capability === selectedRequirement.value!.capability);
    if (cap) return cap.parsed;
  }
  return specBrowserCapabilities.value[0]?.parsed ?? null;
});

const selectedRequirementData = computed(() => {
  if (!selectedRequirement.value) return null;
  const cap = specBrowserCapabilities.value.find((c) => c.capability === selectedRequirement.value!.capability);
  return cap?.parsed.requirements[selectedRequirement.value.index] ?? null;
});

function selectRequirement(capability: string, index: number) {
  selectedRequirement.value = { capability, index };
}

watch(
  () => specBrowserCapabilities.value.map((c) => c.path).join("|"),
  (key, oldKey) => {
    if (key === oldKey) return;
    const first = specBrowserCapabilities.value.find((c) => c.parsed.requirements.length);
    selectedRequirement.value = first ? { capability: first.capability, index: 0 } : null;
  }
);

async function openChangeSpecs(folder: string) {
  const group = allGroupedChanges.value.find((g) => g.folder === folder);
  if (!group || !group.specFiles.length) return;
  activeChangeFolder.value = folder;
  activeSection.value = "changes";
  changeSpecsMode.value = true;
  activeFilePath.value = "";
  activeFileContent.value = "";
  openQuestionsText.value = "";
  const results: SpecBrowserFile[] = [];
  for (const f of group.specFiles) {
    try {
      const content = await invoke<string>("read_file_content", { path: f.path });
      results.push({ capability: f.capability, path: f.path, content, parsed: parseSpec(content) });
    } catch (err) {
      setError("Error reading file: " + (err as Error).message);
    }
  }
  changeSpecsFiles.value = results;
}

async function selectChangeFolder(folder: string) {
  const group = allGroupedChanges.value.find((g) => g.folder === folder);
  if (!group) return;
  const preferred =
    group.tabs.find((t) => t.kind === "proposal") ||
    group.tabs.find((t) => t.kind === "design") ||
    group.tabs.find((t) => t.kind === "tasks") ||
    group.tabs.find((t) => t.kind !== "specs");
  if (preferred) {
    await selectFile({ path: preferred.path, relative: preferred.relative, section: "changes" });
  } else if (group.specFiles.length) {
    await openChangeSpecs(folder);
  }
}

function goToSection(id: string) {
  document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
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
  await projectsStore.upsert(picked);
  await setRoot(picked);
  await loadFolder();
}

async function openProject(path: string) {
  if (path === rootPath.value) return;
  rootPath.value = path;
  await projectsStore.upsert(path);
  await setRoot(path);
  await loadFolder();
}

function goHome() {
  rootPath.value = "";
  specs.value = [];
  changes.value = [];
  activeFilePath.value = "";
  activeFileContent.value = "";
  activeSection.value = "";
  activeChangeFolder.value = "";
  changeSpecsMode.value = false;
  changeSpecsFiles.value = [];
  specRequirementsMode.value = false;
  specViewSection.value = "purpose";
  selectedRequirement.value = null;
  openQuestionsText.value = "";
  errorMsg.value = "";
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
      if (active.section === "changes") {
        activeChangeFolder.value = changeFolderOf(active.relative);
      }
    } else if (changeSpecsMode.value && activeChangeFolder.value) {
      await openChangeSpecs(activeChangeFolder.value);
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
    nextTick(() => fileFilterInput.value?.focus());
  }
}

async function selectFile(entry: { path: string; relative?: string; section?: string }) {
  activeFilePath.value = entry.path;
  activeSection.value = entry.section ?? "";
  viewMode.value = "outline";
  changeSpecsMode.value = false;
  specRequirementsMode.value = false;
  if (entry.section === "changes" && entry.relative) {
    activeChangeFolder.value = changeFolderOf(entry.relative);
  }
  try {
    const content = await invoke<string>("read_file_content", { path: entry.path });
    activeFileContent.value = content;
    extractQuestions(content);
    if (entry.section === "specs") {
      showSpecSection(defaultSpecSection(parseSpec(content)));
    }
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

function parseOpenQuestions(content: string): string {
  const match = content.match(/## Open Questions([\s\S]*?)(?=\n## |$)/);
  return match ? match[1].trim() : "";
}

function withOpenQuestions(content: string, newQuestions: string): string {
  if (content.includes("## Open Questions")) {
    return content.replace(
      /## Open Questions[\s\S]*?(?=\n## |$)/,
      `## Open Questions\n${newQuestions}\n\n`
    );
  }
  return content + `\n\n## Open Questions\n${newQuestions}`;
}

function extractQuestions(content: string) {
  openQuestionsText.value = parseOpenQuestions(content);
}

const activeSpecFile = computed(() => {
  if (!changeSpecsMode.value || !selectedRequirement.value) return null;
  return changeSpecsFiles.value.find((c) => c.capability === selectedRequirement.value!.capability) ?? null;
});

const displayPath = computed(() => (changeSpecsMode.value ? activeSpecFile.value?.path ?? "" : activeFilePath.value));

const displayOpenQuestions = computed(() =>
  changeSpecsMode.value ? parseOpenQuestions(activeSpecFile.value?.content ?? "") : openQuestionsText.value
);

async function saveQuestions(newQuestions: string) {
  if (changeSpecsMode.value && activeSpecFile.value) {
    const file = activeSpecFile.value;
    const content = withOpenQuestions(file.content, newQuestions);
    file.content = content;
    try {
      await invoke("save_file_content", { path: file.path, content });
    } catch (err) {
      setError("Error saving file: " + (err as Error).message);
    }
    return;
  }
  updateContent(withOpenQuestions(activeFileContent.value, newQuestions));
}

function onKeyDown(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey;
  if (mod && (e.key === "f" || e.key === "F")) {
    e.preventDefault();
    if (fileFilterInput.value) {
      fileFilterInput.value.focus();
      fileFilterInput.value.select();
    }
  }
}

let fsChangeTimer: ReturnType<typeof setTimeout> | undefined;

function scheduleReload() {
  clearTimeout(fsChangeTimer);
  fsChangeTimer = setTimeout(loadFolder, 300);
}

onMounted(async () => {
  unlisten.value = await listen("fs-change", scheduleReload);
  window.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  clearTimeout(fsChangeTimer);
  if (unlisten.value) unlisten.value();
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<template>
  <ProjectHome
    v-if="!rootPath"
    @openProject="openProject"
    @pickFolder="pickRoot"
  />
  <div v-else class="flex h-screen bg-slate-950 text-slate-100 font-sans">
    <aside class="w-64 border-r border-slate-800 bg-slate-900/50 p-4 flex flex-col shrink-0">
      <div class="flex gap-2 items-stretch mb-3">
        <ProjectSwitcher
          :currentPath="rootPath"
          @openProject="openProject"
          @pickFolder="pickRoot"
          class="flex-1 min-w-0"
        />
        <button
          @click="goHome"
          class="shrink-0 px-2.5 rounded bg-slate-800/80 border border-slate-700 hover:bg-slate-700/80 text-slate-400 hover:text-slate-200 transition cursor-pointer flex items-center justify-center"
          title="Back to home"
        >
          <svg
            class="w-4 h-4"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 10.5l9-7 9 7V20a1 1 0 01-1 1h-5v-6h-6v6H4a1 1 0 01-1-1v-9.5z" />
          </svg>
        </button>
      </div>

      <input
        v-model="fileFilter"
        ref="fileFilterInput"
        type="text"
        placeholder="Filter specs &amp; changes…"
        class="w-full text-[11px] font-mono px-2 py-1.5 mb-3 rounded bg-slate-900 border border-slate-700 text-slate-200 focus:outline-none focus:border-emerald-600"
      />

      <div class="space-y-4 overflow-y-auto flex-1">
        <div v-if="filteredSpecs.length">
          <button
            @click="collapsedSpecs = !collapsedSpecs"
            class="w-full flex items-center justify-between mb-1 cursor-pointer group"
          >
            <h3 class="text-[10px] font-mono uppercase tracking-wider text-slate-500 group-hover:text-slate-300">
              📄 Specs
            </h3>
            <svg
              class="w-3 h-3 text-slate-500 transition-transform"
              :class="collapsedSpecs ? '' : 'rotate-180'"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
            >
              <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div v-if="!collapsedSpecs" class="space-y-1">
            <button
              v-for="entry in filteredSpecs"
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

        <div v-if="filteredChanges.length">
          <button
            @click="collapsedChanges = !collapsedChanges"
            class="w-full flex items-center justify-between mb-1 cursor-pointer group"
          >
            <h3 class="text-[10px] font-mono uppercase tracking-wider text-slate-500 group-hover:text-slate-300">
              🛠️ Changes
            </h3>
            <svg
              class="w-3 h-3 text-slate-500 transition-transform"
              :class="collapsedChanges ? '' : 'rotate-180'"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
            >
              <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div v-if="!collapsedChanges" class="space-y-1">
            <button
              v-for="group in groupedChangeFolders"
              :key="group.folder"
              @click="selectChangeFolder(group.folder)"
              :class="[
                'w-full flex items-center gap-1.5 text-left text-xs p-2 rounded truncate font-mono transition',
                activeChangeFolder === group.folder
                  ? 'bg-slate-800 text-emerald-400 font-bold'
                  : 'text-slate-400 hover:bg-slate-800/50 hover:text-slate-200',
              ]"
            >
              <span class="shrink-0">📁</span>
              <span class="truncate">{{ group.folder }}</span>
            </button>
          </div>
        </div>

        <p
          v-if="(specs.length || changes.length) && !filteredSpecs.length && !filteredChanges.length"
          class="text-xs font-mono text-slate-500"
        >
          No files match “{{ fileFilter }}”.
        </p>

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

      <div class="pt-3 mt-1 border-t border-slate-800 shrink-0">
        <button
          @click="settingsOpen = true"
          class="w-full flex items-center gap-2 text-left text-xs p-2 rounded font-mono text-slate-400 hover:bg-slate-800/50 hover:text-slate-200 transition cursor-pointer"
          title="Settings"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          <span>Settings</span>
        </button>
      </div>
    </aside>

    <main class="flex-1 overflow-hidden flex flex-col">
      <div
        v-if="errorMsg"
        class="mx-6 mt-6 mb-2 p-3 rounded border border-red-800 bg-red-950/40 text-red-300 text-xs font-mono"
      >
        ⚠️ {{ errorMsg }}
      </div>
      <div v-if="loading" class="mx-6 mt-6 mb-2 text-xs font-mono text-slate-500">Loading…</div>

      <div
        v-if="activeFilePath || (changeSpecsMode && activeChangeGroup)"
        class="sticky top-0 z-20 -mx-0 px-6 py-3 bg-slate-950/95 backdrop-blur border-b border-slate-800"
      >
        <div class="max-w-4xl">
          <div class="flex items-baseline gap-4">
            <h1 class="text-lg font-bold text-slate-100 truncate">{{ activeTitle }}</h1>
          </div>
          <span
            v-if="displayPath"
            class="block text-[10px] font-mono text-slate-500 truncate mt-0.5"
            :title="displayPath"
          >
            {{ displayPath }}
          </span>
          <div class="flex items-center gap-2 mt-2 flex-wrap">
            <div v-if="activeSection === 'changes' && activeChangeGroup" class="flex flex-wrap gap-1.5">
              <button
                v-for="tab in activeChangeGroup.tabs"
                :key="tab.kind === 'specs' ? activeChangeGroup.folder + ':specs' : tab.path"
                @click="tab.kind === 'specs' ? openChangeSpecs(activeChangeGroup.folder) : selectFile({ path: tab.path, relative: tab.relative, section: 'changes' })"
                :class="[
                  'text-[11px] px-2.5 py-1 rounded font-mono transition',
                  (tab.kind === 'specs' ? changeSpecsMode : !changeSpecsMode && activeFilePath === tab.path)
                    ? 'bg-slate-800 text-emerald-400 font-bold'
                    : 'bg-slate-800/60 hover:bg-slate-700 text-slate-300',
                ]"
              >
                {{ tab.icon }} {{ tab.label }}
              </button>
            </div>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-if="outlineParsed?.purpose"
                @click="showSpecSection('purpose')"
                :class="[
                  'text-[11px] px-2.5 py-1 rounded font-mono transition',
                  specViewSection === 'purpose'
                    ? 'bg-slate-800 text-emerald-400 font-bold'
                    : 'bg-slate-800/60 hover:bg-slate-700 text-slate-300',
                ]"
              >
                🎯 Purpose
              </button>
              <button
                v-if="outlineParsed?.requirements.length"
                @click="showSpecSection('requirements')"
                :class="[
                  'text-[11px] px-2.5 py-1 rounded font-mono transition',
                  specViewSection === 'requirements'
                    ? 'bg-slate-800 text-emerald-400 font-bold'
                    : 'bg-slate-800/60 hover:bg-slate-700 text-slate-300',
                ]"
              >
                📌 Requirements ({{ outlineParsed.requirements.length }})
              </button>
              <button
                v-if="outlineParsed?.topScenarios.length"
                @click="showSpecSection('scenarios')"
                :class="[
                  'text-[11px] px-2.5 py-1 rounded font-mono transition',
                  specViewSection === 'scenarios'
                    ? 'bg-slate-800 text-emerald-400 font-bold'
                    : 'bg-slate-800/60 hover:bg-slate-700 text-slate-300',
                ]"
              >
                🎬 Scenarios ({{ outlineParsed.topScenarios.length }})
              </button>
              <button
                v-if="outlineParsed?.limitations"
                @click="showSpecSection('limitations')"
                :class="[
                  'text-[11px] px-2.5 py-1 rounded font-mono transition',
                  specViewSection === 'limitations'
                    ? 'bg-slate-800 text-emerald-400 font-bold'
                    : 'bg-slate-800/60 hover:bg-slate-700 text-slate-300',
                ]"
              >
                ⚠️ Known Limitations
              </button>
              <button
                v-if="outlineParsed?.nonGoals"
                @click="showSpecSection('nonGoals')"
                :class="[
                  'text-[11px] px-2.5 py-1 rounded font-mono transition',
                  specViewSection === 'nonGoals'
                    ? 'bg-slate-800 text-emerald-400 font-bold'
                    : 'bg-slate-800/60 hover:bg-slate-700 text-slate-300',
                ]"
              >
                🚫 Non-Goals
              </button>
              <button
                @click="onOpenQuestionsClick"
                :class="[
                  'text-[11px] px-2.5 py-1 rounded font-mono transition',
                  activeSection === 'specs' && viewMode === 'outline' && specViewSection === 'openQuestions'
                    ? 'bg-slate-800 text-emerald-400 font-bold'
                    : 'bg-slate-800/60 hover:bg-slate-700 text-amber-300',
                ]"
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

      <div class="flex-1 min-h-0 flex overflow-hidden">
        <div
          v-if="activeSection === 'specs' && viewMode === 'outline'"
          class="flex-1 min-h-0 overflow-y-auto px-6 py-6 max-w-4xl"
        >
          <section v-if="specViewSection === 'purpose' && outlineParsed?.purpose">
            <h3 class="text-xs font-mono uppercase tracking-wider text-emerald-400 mb-2">🎯 Purpose</h3>
            <div
              class="text-sm text-slate-200 leading-relaxed prose prose-invert max-w-none"
              v-html="renderMd(outlineParsed.purpose)"
            />
          </section>

          <div v-else-if="specViewSection === 'requirements'">
            <div v-if="selectedRequirementData">
              <h2 class="text-lg font-bold text-slate-100 mb-3">📌 {{ selectedRequirementData.name }}</h2>
              <div
                v-if="selectedRequirementData.description"
                class="text-sm text-slate-200 leading-relaxed prose prose-invert max-w-none mb-4"
                v-html="renderMd(selectedRequirementData.description)"
              />
              <div v-if="selectedRequirementData.scenarios.length" class="space-y-2">
                <h3 class="text-xs font-mono uppercase tracking-wider text-cyan-400 mb-2">🎬 Scenarios</h3>
                <div
                  v-for="(sc, j) in selectedRequirementData.scenarios"
                  :key="j"
                  class="rounded-lg border border-slate-800 bg-slate-900/60 p-3"
                >
                  <div class="text-xs font-mono text-cyan-300 mb-1">{{ sc.name }}</div>
                  <div class="text-sm text-slate-300 prose prose-invert max-w-none" v-html="renderMd(sc.body)" />
                </div>
              </div>
            </div>
            <p v-else class="text-xs text-slate-500">Select a requirement to see its details.</p>
          </div>

          <section v-else-if="specViewSection === 'scenarios' && outlineParsed?.topScenarios.length">
            <h3 class="text-xs font-mono uppercase tracking-wider text-cyan-400 mb-2">🎬 Scenarios</h3>
            <div class="space-y-2">
              <div
                v-for="(sc, j) in outlineParsed.topScenarios"
                :key="j"
                class="rounded-lg border border-slate-800 bg-slate-900/60 p-3"
              >
                <div class="text-xs font-mono text-cyan-300 mb-1">{{ sc.name }}</div>
                <div class="text-sm text-slate-300 prose prose-invert max-w-none" v-html="renderMd(sc.body)" />
              </div>
            </div>
          </section>

          <section v-else-if="specViewSection === 'limitations' && outlineParsed?.limitations">
            <h3 class="text-xs font-mono uppercase tracking-wider text-amber-400 mb-2">⚠️ Known Limitations</h3>
            <div
              class="text-sm text-slate-200 prose prose-invert max-w-none"
              v-html="renderMd(outlineParsed.limitations)"
            />
          </section>

          <section v-else-if="specViewSection === 'nonGoals' && outlineParsed?.nonGoals">
            <h3 class="text-xs font-mono uppercase tracking-wider text-slate-400 mb-2">🚫 Non-Goals</h3>
            <div
              class="text-sm text-slate-300 prose prose-invert max-w-none"
              v-html="renderMd(outlineParsed.nonGoals)"
            />
          </section>

          <div v-else-if="specViewSection === 'openQuestions'">
            <OpenQuestions :questionsText="openQuestionsText" @saveQuestions="saveQuestions" />
          </div>
        </div>

        <div v-else-if="changeSpecsMode" class="flex-1 min-h-0 overflow-y-auto px-6 py-6 max-w-4xl space-y-6">
          <section v-if="specBrowserOverview?.purpose" id="o-purpose" class="scroll-mt-24">
            <h3 class="text-xs font-mono uppercase tracking-wider text-emerald-400 mb-2">🎯 Purpose</h3>
            <div
              class="text-sm text-slate-200 leading-relaxed prose prose-invert max-w-none"
              v-html="renderMd(specBrowserOverview.purpose)"
            />
          </section>

          <div>
            <div v-if="selectedRequirementData">
              <h2 class="text-lg font-bold text-slate-100 mb-3">📌 {{ selectedRequirementData.name }}</h2>
              <div
                v-if="selectedRequirementData.description"
                class="text-sm text-slate-200 leading-relaxed prose prose-invert max-w-none mb-4"
                v-html="renderMd(selectedRequirementData.description)"
              />
              <div v-if="selectedRequirementData.scenarios.length" class="space-y-2">
                <h3 class="text-xs font-mono uppercase tracking-wider text-cyan-400 mb-2">🎬 Scenarios</h3>
                <div
                  v-for="(sc, j) in selectedRequirementData.scenarios"
                  :key="j"
                  class="rounded-lg border border-slate-800 bg-slate-900/60 p-3"
                >
                  <div class="text-xs font-mono text-cyan-300 mb-1">{{ sc.name }}</div>
                  <div class="text-sm text-slate-300 prose prose-invert max-w-none" v-html="renderMd(sc.body)" />
                </div>
              </div>
            </div>
            <p v-else class="text-xs text-slate-500">Select a requirement to see its details.</p>
          </div>

          <section v-if="specBrowserOverview?.topScenarios.length" id="o-scen" class="scroll-mt-24">
            <h3 class="text-xs font-mono uppercase tracking-wider text-cyan-400 mb-2">🎬 Scenarios</h3>
            <div class="space-y-2">
              <div
                v-for="(sc, j) in specBrowserOverview.topScenarios"
                :key="j"
                class="rounded-lg border border-slate-800 bg-slate-900/60 p-3"
              >
                <div class="text-xs font-mono text-cyan-300 mb-1">{{ sc.name }}</div>
                <div class="text-sm text-slate-300 prose prose-invert max-w-none" v-html="renderMd(sc.body)" />
              </div>
            </div>
          </section>

          <section v-if="specBrowserOverview?.limitations" id="o-lim" class="scroll-mt-24">
            <h3 class="text-xs font-mono uppercase tracking-wider text-amber-400 mb-2">⚠️ Known Limitations</h3>
            <div
              class="text-sm text-slate-200 prose prose-invert max-w-none"
              v-html="renderMd(specBrowserOverview.limitations)"
            />
          </section>

          <section v-if="specBrowserOverview?.nonGoals" class="scroll-mt-24">
            <h3 class="text-xs font-mono uppercase tracking-wider text-slate-400 mb-2">🚫 Non-Goals</h3>
            <div
              class="text-sm text-slate-300 prose prose-invert max-w-none"
              v-html="renderMd(specBrowserOverview.nonGoals)"
            />
          </section>

          <div id="open-questions" class="scroll-mt-28">
            <OpenQuestions :questionsText="displayOpenQuestions" @saveQuestions="saveQuestions" />
          </div>
        </div>

        <div v-else-if="activeFileContent" class="flex-1 min-h-0 overflow-y-auto px-6 py-6 max-w-4xl">
          <TaskList :rawContent="activeFileContent" @updateContent="updateContent" />

          <div class="mt-8">
            <SpecDetail :content="activeFileContent" />
          </div>

          <div id="open-questions" class="mt-6 scroll-mt-28">
            <OpenQuestions :questionsText="openQuestionsText" @saveQuestions="saveQuestions" />
          </div>
        </div>

        <ChangeRequirementsSidebar
          v-if="requirementsMode"
          :capabilities="specBrowserCapabilities"
          :selected="selectedRequirement"
          @select="selectRequirement"
        />
      </div>
    </main>

    <SettingsModal v-if="settingsOpen" @close="settingsOpen = false" />
  </div>
</template>
