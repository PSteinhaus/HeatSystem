<script setup lang="ts">
import { ref } from 'vue'
import { gameState, connectName } from '../store'

const emit = defineEmits<{ (e: 'close'): void }>()

const name = ref('')
const error = ref('')

function submit() {
  const trimmed = name.value.trim()
  if (!trimmed) {
    error.value = 'Bitte einen Namen eingeben.'
    return
  }
  if (trimmed.length > 20) {
    error.value = 'Name ist zu lang (max. 20 Zeichen).'
    return
  }
  error.value = ''
  const ok = connectName(trimmed)
  if (ok) {
    emit('close')
  } else {
    error.value = 'Beitritt fehlgeschlagen.'
  }
}

const existingNames = ref<string[]>([])
function refreshExisting() {
  existingNames.value = Object.keys(gameState.players)
}
refreshExisting()
</script>

<template>
  <div class="modal-overlay">
    <div class="modal name-modal">
      <h2>Wähle deinen Namen</h2>
      <p class="desc">Unter diesem Namen werden deine Werte gespeichert. Verlässt du die Seite und kommst zurück, gib einfach denselben Namen ein.</p>

      <form @submit.prevent="submit">
        <input
          ref="inputEl"
          v-model="name"
          placeholder="Dein Name"
          maxlength="20"
          autofocus
        />
        <div v-if="error" class="error">{{ error }}</div>
        <button class="primary" type="submit">Beitreten</button>
      </form>

      <div v-if="existingNames.length" class="existing">
        <p class="existing-label">Bereits dabei:</p>
        <div class="chips">
          <span v-for="n in existingNames" :key="n" class="chip" @click="name = n">{{ n }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.name-modal {
  max-width: 380px;
  width: 90vw;
}
h2 {
  margin: 0 0 0.5rem;
  font-size: 1.3rem;
}
.desc {
  color: var(--text-muted);
  font-size: 0.85rem;
  margin: 0 0 1.5rem;
  line-height: 1.5;
}
form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
form input {
  width: 100%;
}
.error {
  color: var(--error);
  font-size: 0.85rem;
}
form button {
  width: 100%;
  padding: 0.7em;
}
.existing {
  margin-top: 1.5rem;
  padding-top: 1.25rem;
  border-top: 1px solid var(--border);
}
.existing-label {
  color: var(--text-dim);
  font-size: 0.8rem;
  margin: 0 0 0.5rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}
.chip {
  background: var(--bg-elevated);
  border: 1px solid var(--border-light);
  border-radius: 999px;
  padding: 0.3em 0.8em;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.2s;
}
.chip:hover {
  border-color: var(--primary-dim);
  background: var(--bg-hover);
}
</style>
