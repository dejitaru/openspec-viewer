<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ questionsText: string }>();
const emit = defineEmits<{ (e: "saveQuestions", questions: string): void }>();
const localText = ref(props.questionsText);
const isEditing = ref(false);

watch(
  () => props.questionsText,
  (newVal) => {
    if (!isEditing.value) {
      localText.value = newVal;
    }
  }
);

function save() {
  emit("saveQuestions", localText.value);
  isEditing.value = false;
}
</script>

<template>
  <div class="my-6 p-4 bg-amber-950/20 border border-amber-900/40 rounded-lg">
    <div class="flex justify-between items-center mb-3">
      <h3 class="text-sm font-semibold text-amber-400 flex items-center gap-2">
        <span>❓</span> Open Questions
      </h3>
      <button
        @click="isEditing ? save() : (isEditing = true)"
        class="text-xs px-3 py-1 rounded bg-amber-900/50 hover:bg-amber-900 text-amber-200 font-mono transition"
      >
        {{ isEditing ? "💾 Save" : "✏️ Edit" }}
      </button>
    </div>

    <div
      v-if="!isEditing"
      class="text-sm text-amber-100/80 whitespace-pre-wrap font-sans leading-relaxed"
    >
      {{ localText || "No open questions recorded in this file." }}
    </div>

    <textarea
      v-else
      v-model="localText"
      rows="5"
      class="w-full bg-slate-950 border border-amber-800/50 rounded p-3 text-sm text-amber-100 font-mono focus:outline-none focus:border-amber-500"
    ></textarea>
  </div>
</template>
