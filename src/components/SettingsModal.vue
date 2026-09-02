<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useProjectsStore, humanizeProjectName } from "../stores/projects";

const props = defineProps<{ projectPath: string }>();
const emit = defineEmits<{
  (e: "close"): void;
  (e: "unlinked"): void;
}>();

const store = useProjectsStore();

const projectName = computed(() => {
  const p = store.projects.find((p) => p.path === props.projectPath);
  return p?.name ?? humanizeProjectName(props.projectPath);
});

const unlinkConfirmText = ref("");
const canUnlink = computed(
  () => unlinkConfirmText.value.trim() === projectName.value
);

type SettingsTab = "theme" | "project";
const activeTab = ref<SettingsTab>("theme");

onMounted(() => {
  store.loadThemes();
});

async function selectTheme(id: string) {
  await store.setTheme(id);
}

async function openConfigFolder() {
  try {
    const path = await invoke<string>("storage_path");
    await revealItemInDir(path);
  } catch (err) {
    console.error("openConfigFolder", err);
  }
}

async function unlinkProject() {
  if (!canUnlink.value) return;
  await store.remove(props.projectPath);
  unlinkConfirmText.value = "";
  emit("unlinked");
  emit("close");
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60" @click.self="emit('close')">
    <div class="w-96 max-h-[85vh] overflow-y-auto rounded-lg border border-slate-800 bg-slate-900 p-4 shadow-xl">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-sm font-bold text-slate-100">⚙️ Settings</h2>
        <button
          @click="emit('close')"
          class="text-slate-500 hover:text-slate-200 transition cursor-pointer"
          title="Close"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="flex gap-1 mb-4 border-b border-slate-800">
        <button
          @click="activeTab = 'theme'"
          :class="[
            'px-3 py-1.5 text-xs font-mono border-b-2 -mb-px transition',
            activeTab === 'theme'
              ? 'border-emerald-500 text-emerald-400 font-bold'
              : 'border-transparent text-slate-500 hover:text-slate-300',
          ]"
        >
          Theme
        </button>
        <button
          @click="activeTab = 'project'"
          :class="[
            'px-3 py-1.5 text-xs font-mono border-b-2 -mb-px transition',
            activeTab === 'project'
              ? 'border-red-500 text-red-400 font-bold'
              : 'border-transparent text-slate-500 hover:text-slate-300',
          ]"
        >
          Project
        </button>
      </div>

      <div v-if="activeTab === 'theme'">
        <h3 class="text-[10px] font-mono uppercase tracking-wider text-slate-500 mb-2">Theme</h3>
        <div class="space-y-1.5 mb-4">
          <button
            v-for="t in store.themes"
            :key="t.id"
            @click="selectTheme(t.id)"
            :class="[
              'w-full flex items-center gap-2.5 text-left text-xs p-2 rounded font-mono transition',
              store.theme === t.id
                ? 'bg-emerald-500/15 ring-1 ring-inset ring-emerald-500'
                : 'text-slate-300 hover:bg-slate-800/50',
            ]"
          >
            <span :class="store.theme === t.id ? 'flex-1 text-slate-100 font-bold' : 'flex-1'">{{ t.label }}</span>
            <span
              :class="[
                'w-3.5 h-3.5 rounded-full shrink-0 border flex items-center justify-center',
                store.theme === t.id ? 'border-emerald-500 bg-emerald-500' : 'border-slate-600',
              ]"
            >
              <svg
                v-if="store.theme === t.id"
                xmlns="http://www.w3.org/2000/svg"
                class="w-2.5 h-2.5 text-slate-950"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width="3"
              >
                <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
              </svg>
            </span>
          </button>
        </div>

        <p class="text-[11px] text-slate-500 leading-relaxed mb-2">
          Add a theme by copying a JSON file into the <span class="font-mono">themes</span> subfolder — duplicate an
          existing one and edit its colors.
        </p>
        <button
          @click="openConfigFolder"
          class="w-full flex items-center gap-2 text-left text-xs p-2 rounded font-mono bg-slate-800/60 hover:bg-slate-700 text-slate-300 transition"
        >
          📁 Open config folder
        </button>
      </div>

      <div v-else>
        <h3 class="text-[10px] font-mono uppercase tracking-wider text-red-400 mb-2">⚠️ Danger zone</h3>
        <p class="text-[11px] text-slate-500 leading-relaxed mb-2">
          Unlink <span class="font-mono text-slate-300">{{ projectName }}</span> from this app. Your files on disk
          are not touched — this only removes it from the project list.
        </p>
        <p class="text-[11px] text-slate-500 mb-1.5">
          Type <span class="font-mono text-red-300">{{ projectName }}</span> to confirm:
        </p>
        <input
          v-model="unlinkConfirmText"
          type="text"
          :placeholder="projectName"
          class="w-full text-xs font-mono px-2 py-1.5 mb-2 rounded bg-slate-950 border border-red-950/80 text-slate-200 focus:outline-none focus:border-red-600"
          @keydown.enter="unlinkProject"
        />
        <button
          @click="unlinkProject"
          :disabled="!canUnlink"
          class="w-full text-left text-xs p-2 rounded font-mono transition"
          :class="
            canUnlink
              ? 'bg-red-950/60 text-red-300 hover:bg-red-900/60 cursor-pointer'
              : 'bg-slate-800/40 text-slate-600 cursor-not-allowed'
          "
        >
          🗑️ Unlink project
        </button>
      </div>
    </div>
  </div>
</template>
