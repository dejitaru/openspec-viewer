<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";
import { useProjectsStore, humanizeProjectName } from "../stores/projects";
import type { Project } from "../stores/projects";

const props = defineProps<{ currentPath: string }>();
const emit = defineEmits<{
  (e: "openProject", path: string): void;
  (e: "pickFolder"): void;
}>();

const store = useProjectsStore();

const dropdownOpen = ref(false);
const contextMenu = ref<{ x: number; y: number; project: Project } | null>(null);
const editing = ref<{ project: Project } | null>(null);
const editIcon = ref("");
const editName = ref("");
const rootEl = ref<HTMLElement | null>(null);

const current = computed(
  () => store.projects.find((p) => p.path === props.currentPath) ?? null
);

const hasProjects = computed(
  () => props.currentPath !== "" || store.projects.length > 0
);

const displayName = computed(() => {
  if (current.value?.name) return current.value.name;
  if (props.currentPath) return humanizeProjectName(props.currentPath);
  return "No project";
});

const displayIcon = computed(() => current.value?.icon ?? "📂");

onMounted(async () => {
  if (!store.loaded) await store.load();
  if (props.currentPath) await store.upsert(props.currentPath);
  document.addEventListener("click", onDocClick);
  document.addEventListener("contextmenu", onDocClick);
});

function toggleDropdown() {
  if (store.projects.length <= 1) return;
  dropdownOpen.value = !dropdownOpen.value;
  contextMenu.value = null;
  editing.value = null;
}

function selectProject(path: string) {
  dropdownOpen.value = false;
  if (path !== props.currentPath) emit("openProject", path);
}

function openContextMenu(e: MouseEvent, project: Project) {
  e.preventDefault();
  e.stopPropagation();
  dropdownOpen.value = false;
  editing.value = null;
  contextMenu.value = { x: e.clientX, y: e.clientY, project };
}

function startEdit() {
  if (!contextMenu.value) return;
  editing.value = { project: contextMenu.value.project };
  editIcon.value = contextMenu.value.project.icon;
  editName.value = contextMenu.value.project.name;
  contextMenu.value = null;
  nextTick(() => {
    const el = document.querySelector<HTMLInputElement>(".ps-edit-input");
    el?.focus();
  });
}

async function saveEdit() {
  if (!editing.value) return;
  await store.update(editing.value.project.path, {
    icon: editIcon.value,
    name: editName.value,
  });
  editing.value = null;
}

async function removeProject() {
  if (!contextMenu.value) return;
  await store.remove(contextMenu.value.project.path);
  contextMenu.value = null;
}

function closeAll() {
  dropdownOpen.value = false;
  contextMenu.value = null;
  editing.value = null;
}

function onDocClick(e: MouseEvent) {
  if (rootEl.value && !rootEl.value.contains(e.target as Node)) closeAll();
}

onUnmounted(() => {
  document.removeEventListener("click", onDocClick);
  document.removeEventListener("contextmenu", onDocClick);
});
</script>

<template>
  <div ref="rootEl" v-if="hasProjects" class="relative select-none">
    <button
      @click="toggleDropdown"
      @contextmenu="openContextMenu($event, current ?? { path: props.currentPath, name: displayName, icon: displayIcon })"
      class="w-full flex items-center gap-2 px-3 py-2 rounded bg-slate-800/80 border border-slate-700 hover:bg-slate-700/80 text-left transition"
      :title="props.currentPath"
    >
      <span class="text-base leading-none">{{ displayIcon }}</span>
      <span class="flex-1 min-w-0 truncate text-xs font-mono text-slate-200">
        {{ displayName }}
      </span>
      <svg
        v-if="store.projects.length > 1"
        class="w-3.5 h-3.5 text-slate-500 shrink-0 transition-transform"
        :class="dropdownOpen ? 'rotate-180' : ''"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        stroke-width="2"
      >
        <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <div
      v-if="dropdownOpen"
      class="absolute left-0 right-0 top-full mt-1 z-50 rounded bg-slate-900 border border-slate-700 shadow-xl shadow-black/40 overflow-hidden"
    >
      <div class="max-h-64 overflow-y-auto">
        <button
          v-for="p in store.projects"
          :key="p.path"
          @click="selectProject(p.path)"
          @contextmenu="openContextMenu($event, p)"
          :class="[
            'w-full flex items-center gap-2 px-3 py-2 text-xs font-mono text-left transition',
            p.path === props.currentPath
              ? 'bg-slate-800 text-emerald-300'
              : 'text-slate-300 hover:bg-slate-800/70',
          ]"
          :title="p.path"
        >
          <span class="text-sm leading-none">{{ p.icon }}</span>
          <span class="flex-1 min-w-0 truncate">{{ p.name }}</span>
          <span v-if="p.path === props.currentPath" class="text-emerald-400 text-[10px]">●</span>
        </button>
        <p v-if="!store.projects.length" class="px-3 py-2 text-[10px] font-mono text-slate-500">
          No saved projects yet.
        </p>
      </div>
      <button
        @click="emit('pickFolder'); dropdownOpen = false"
        class="w-full px-3 py-2 text-[10px] font-mono text-emerald-300 text-left bg-slate-900 border-t border-slate-700 hover:bg-slate-800 transition"
      >
        ＋ Select another folder…
      </button>
    </div>

    <div
      v-if="contextMenu"
      class="fixed z-[60] min-w-40 rounded bg-slate-900 border border-slate-700 shadow-xl shadow-black/40 py-1"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @contextmenu.prevent
    >
      <button
        @click="startEdit"
        class="w-full px-3 py-1.5 text-xs font-mono text-slate-200 text-left hover:bg-slate-800 transition"
      >
        ✏️ Edit icon &amp; name
      </button>
      <button
        @click="removeProject"
        class="w-full px-3 py-1.5 text-xs font-mono text-red-300 text-left hover:bg-slate-800 transition"
      >
        🗑️ Remove from list
      </button>
    </div>

    <div
      v-if="editing"
      class="fixed z-[70] w-56 rounded bg-slate-900 border border-slate-700 shadow-xl shadow-black/40 p-3"
      :style="{ left: contextMenu?.x ?? '50%', top: (contextMenu?.y ?? 0) + 'px' }"
      @click.stop
    >
      <label class="block text-[10px] font-mono text-slate-400 mb-1">Icon (emoji)</label>
      <input
        v-model="editIcon"
        class="ps-edit-input w-full text-xs font-mono px-2 py-1 mb-2 rounded bg-slate-950 border border-slate-700 text-slate-200 focus:outline-none focus:border-emerald-600"
        placeholder="📂"
      />
      <label class="block text-[10px] font-mono text-slate-400 mb-1">Name</label>
      <input
        v-model="editName"
        class="w-full text-xs font-mono px-2 py-1 mb-3 rounded bg-slate-950 border border-slate-700 text-slate-200 focus:outline-none focus:border-emerald-600"
        placeholder="Project name"
        @keydown.enter="saveEdit"
        @keydown.esc="editing = null"
      />
      <div class="flex justify-end gap-2">
        <button
          @click="editing = null"
          class="text-[10px] font-mono px-2 py-1 rounded bg-slate-800 text-slate-300 hover:bg-slate-700 transition"
        >
          Cancel
        </button>
        <button
          @click="saveEdit"
          class="text-[10px] font-mono px-2 py-1 rounded bg-emerald-700 text-white hover:bg-emerald-600 transition"
        >
          Save
        </button>
      </div>
    </div>
  </div>
</template>
