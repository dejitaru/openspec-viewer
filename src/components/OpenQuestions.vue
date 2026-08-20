<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ questionsText: string }>();
const emit = defineEmits<{ (e: "saveQuestions", questions: string): void }>();

interface QuestionItem {
  id: number;
  text: string;
}

let nextId = 0;
const questions = ref<QuestionItem[]>([]);
const editingId = ref<number | null>(null);
const draftText = ref("");
const addingNew = ref(false);
const newText = ref("");

function parseQuestions(raw: string): QuestionItem[] {
  const items: QuestionItem[] = [];
  for (const line of raw.split("\n")) {
    const match = line.match(/^\s*[-*]\s+(.*)$/);
    if (match && match[1].trim()) {
      items.push({ id: nextId++, text: match[1].trim() });
    }
  }
  if (!items.length && raw.trim()) {
    items.push({ id: nextId++, text: raw.trim() });
  }
  return items;
}

watch(
  () => props.questionsText,
  (raw) => {
    editingId.value = null;
    addingNew.value = false;
    questions.value = parseQuestions(raw);
  },
  { immediate: true }
);

function commit() {
  emit(
    "saveQuestions",
    questions.value.map((q) => `- ${q.text}`).join("\n")
  );
}

function startEdit(item: QuestionItem) {
  addingNew.value = false;
  editingId.value = item.id;
  draftText.value = item.text;
}

function saveEdit() {
  const item = questions.value.find((q) => q.id === editingId.value);
  if (item) {
    const trimmed = draftText.value.trim();
    if (trimmed) {
      item.text = trimmed;
    } else {
      questions.value = questions.value.filter((q) => q.id !== item.id);
    }
  }
  editingId.value = null;
  commit();
}

function cancelEdit() {
  editingId.value = null;
}

function removeQuestion(item: QuestionItem) {
  questions.value = questions.value.filter((q) => q.id !== item.id);
  commit();
}

function startAdd() {
  editingId.value = null;
  addingNew.value = true;
  newText.value = "";
}

function saveAdd() {
  const trimmed = newText.value.trim();
  if (trimmed) {
    questions.value.push({ id: nextId++, text: trimmed });
    commit();
  }
  addingNew.value = false;
  newText.value = "";
}

function cancelAdd() {
  addingNew.value = false;
  newText.value = "";
}
</script>

<template>
  <div class="my-6 p-4 bg-amber-950/20 border border-amber-900/40 rounded-lg">
    <div class="flex justify-between items-center mb-3">
      <h3 class="text-sm font-semibold text-amber-400 flex items-center gap-2">
        <span>❓</span> Open Questions
        <span class="text-[10px] font-mono px-1.5 py-0.5 rounded-full bg-amber-900/50 text-amber-300">
          {{ questions.length }}
        </span>
      </h3>
      <button
        v-if="!addingNew"
        @click="startAdd"
        class="text-xs px-3 py-1 rounded bg-amber-900/50 hover:bg-amber-900 text-amber-200 font-mono transition"
      >
        ＋ Add
      </button>
    </div>

    <p v-if="!questions.length && !addingNew" class="text-sm text-amber-100/60">
      No open questions recorded in this file.
    </p>

    <div class="space-y-2">
      <div
        v-for="(item, idx) in questions"
        :key="item.id"
        class="group rounded-md border border-amber-900/40 bg-slate-950/40 p-3"
      >
        <div v-if="editingId === item.id" class="space-y-2">
          <textarea
            v-model="draftText"
            rows="3"
            autofocus
            class="w-full bg-slate-950 border border-amber-800/50 rounded p-2 text-sm text-amber-100 font-mono focus:outline-none focus:border-amber-500"
            @keydown.meta.enter="saveEdit"
            @keydown.ctrl.enter="saveEdit"
            @keydown.esc="cancelEdit"
          ></textarea>
          <div class="flex gap-2">
            <button
              @click="saveEdit"
              class="text-xs px-2.5 py-1 rounded bg-amber-900/50 hover:bg-amber-900 text-amber-200 font-mono transition"
            >
              💾 Save
            </button>
            <button
              @click="cancelEdit"
              class="text-xs px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono transition"
            >
              Cancel
            </button>
          </div>
        </div>
        <div v-else class="flex items-start gap-2">
          <span class="text-[10px] font-mono text-amber-500/70 mt-0.5 shrink-0">{{ idx + 1 }}.</span>
          <p class="flex-1 text-sm text-amber-100/80 whitespace-pre-wrap font-sans leading-relaxed">
            {{ item.text }}
          </p>
          <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition shrink-0">
            <button
              @click="startEdit(item)"
              title="Edit"
              class="text-xs px-1.5 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 transition"
            >
              ✏️
            </button>
            <button
              @click="removeQuestion(item)"
              title="Remove"
              class="text-xs px-1.5 py-0.5 rounded bg-slate-800 hover:bg-red-900 text-slate-300 hover:text-red-200 transition"
            >
              🗑️
            </button>
          </div>
        </div>
      </div>

      <div v-if="addingNew" class="rounded-md border border-amber-900/40 bg-slate-950/40 p-3 space-y-2">
        <textarea
          v-model="newText"
          rows="3"
          autofocus
          placeholder="Write the new question…"
          class="w-full bg-slate-950 border border-amber-800/50 rounded p-2 text-sm text-amber-100 font-mono focus:outline-none focus:border-amber-500"
          @keydown.meta.enter="saveAdd"
          @keydown.ctrl.enter="saveAdd"
          @keydown.esc="cancelAdd"
        ></textarea>
        <div class="flex gap-2">
          <button
            @click="saveAdd"
            class="text-xs px-2.5 py-1 rounded bg-amber-900/50 hover:bg-amber-900 text-amber-200 font-mono transition"
          >
            💾 Save
          </button>
          <button
            @click="cancelAdd"
            class="text-xs px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono transition"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
