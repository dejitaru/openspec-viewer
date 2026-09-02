<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  root: string;
  folder: string;
  label: string;
}>();
const emit = defineEmits<{
  (e: "close"): void;
  (e: "deleted"): void;
}>();

const confirmText = ref("");
const deleteFromDisk = ref(false);
const busy = ref(false);
const errorMsg = ref("");
const inputEl = ref<HTMLInputElement | null>(null);

const canDelete = computed(() => confirmText.value.trim() === props.label && !busy.value);

onMounted(() => {
  nextTick(() => inputEl.value?.focus());
});

async function confirmDelete() {
  if (!canDelete.value) return;
  busy.value = true;
  errorMsg.value = "";
  try {
    if (deleteFromDisk.value) {
      await invoke("delete_change", { root: props.root, folder: props.folder });
    } else {
      await invoke("trash_change", { root: props.root, folder: props.folder });
    }
    emit("deleted");
    emit("close");
  } catch (err) {
    errorMsg.value = String(err);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60" @click.self="emit('close')">
    <div class="w-96 rounded-lg border border-red-950/60 bg-slate-900 p-4 shadow-xl">
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-sm font-bold text-slate-100">🗑️ Delete change</h2>
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

      <p class="text-[11px] text-slate-500 leading-relaxed mb-3">
        By default this moves <span class="font-mono text-slate-300">{{ label }}</span> to a hidden
        <span class="font-mono">.trash</span> folder inside <span class="font-mono">openspec/changes/</span> — it
        disappears from the app but stays recoverable on disk. It is <em>not</em> moved to
        <span class="font-mono">changes/archive/</span>, since we can't confirm the change was actually implemented.
      </p>

      <label class="flex items-start gap-2 mb-3 cursor-pointer select-none">
        <input v-model="deleteFromDisk" type="checkbox" class="mt-0.5 accent-red-600" />
        <span class="text-[11px] text-slate-400 leading-relaxed">
          Also delete permanently from disk instead of trashing —
          <span class="text-red-400 font-semibold">this cannot be undone</span>.
        </span>
      </label>

      <p class="text-[11px] text-slate-500 mb-1.5">
        Type <span class="font-mono text-red-300">{{ label }}</span> to confirm:
      </p>
      <input
        ref="inputEl"
        v-model="confirmText"
        type="text"
        :placeholder="label"
        class="w-full text-xs font-mono px-2 py-1.5 mb-3 rounded bg-slate-950 border border-red-950/80 text-slate-200 focus:outline-none focus:border-red-600"
        @keydown.enter="confirmDelete"
        @keydown.esc="emit('close')"
      />

      <p v-if="errorMsg" class="text-[11px] text-red-400 mb-2">⚠️ {{ errorMsg }}</p>

      <div class="flex justify-end gap-2">
        <button
          @click="emit('close')"
          class="text-xs font-mono px-3 py-1.5 rounded bg-slate-800 text-slate-300 hover:bg-slate-700 transition"
        >
          Cancel
        </button>
        <button
          @click="confirmDelete"
          :disabled="!canDelete"
          class="text-xs font-mono px-3 py-1.5 rounded transition"
          :class="
            canDelete
              ? 'bg-red-950/60 text-red-300 hover:bg-red-900/60 cursor-pointer'
              : 'bg-slate-800/40 text-slate-600 cursor-not-allowed'
          "
        >
          {{ busy ? "Deleting…" : deleteFromDisk ? "🗑️ Delete permanently" : "🗑️ Move to trash" }}
        </button>
      </div>
    </div>
  </div>
</template>
