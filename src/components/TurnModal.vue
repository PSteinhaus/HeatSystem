<script setup lang="ts">
import { computed, ref } from 'vue'
import { beginTurn, myPlayer } from '../store'
import { bonusCost } from '../game/logic'
import { MAX_BONUS } from '../game/constants'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ (e: 'close'): void }>()

const bonus = ref(0)

const myHope = computed(() => myPlayer.value?.hope ?? 0)
const canAfford = computed(() => myPlayer.value ? myPlayer.value.hope >= bonusCost(bonus.value) : false)
const cost = computed(() => bonusCost(bonus.value))

function confirm() {
  if (!canAfford.value || bonus.value < 0) return
  beginTurn(bonus.value)
  emit('close')
}

function cancel() {
  bonus.value = 0
  emit('close')
}
</script>

<template>
  <div v-if="props.open" class="modal-overlay" @click.self="cancel">
    <div class="modal turn-modal">
      <h2>Zug beginnen</h2>
      <p class="desc">Wähle deinen Bonus. Die Kosten betragen B² Hoffnung.</p>

      <div class="bonus-section">
        <div class="bonus-row">
          <button class="step-btn" :disabled="bonus <= 0" @click="bonus--">−</button>
          <div class="bonus-display">
            <span class="bonus-label">Bonus</span>
            <span class="bonus-value">+{{ bonus }}</span>
          </div>
          <button class="step-btn" :disabled="bonus >= MAX_BONUS" @click="bonus++">+</button>
        </div>

        <div class="cost-info">
          <div class="cost-row">
            <span>Kosten</span>
            <span class="cost-value">{{ cost }} Hoffnung</span>
          </div>
          <div class="cost-row">
            <span>Deine Hoffnung</span>
            <span :class="{ insufficient: !canAfford }">{{ myHope }} Hoffnung</span>
          </div>
          <div class="cost-row" v-if="!canAfford">
            <span class="insufficient">Nicht genug Hoffnung!</span>
          </div>
        </div>
      </div>

      <div class="actions">
        <button @click="cancel">Abbrechen</button>
        <button class="primary" :disabled="!canAfford" @click="confirm">Bestätigen</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.turn-modal {
  max-width: 360px;
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
}
.bonus-section {
  margin-bottom: 1.5rem;
}
.bonus-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1.5rem;
  margin-bottom: 1.25rem;
}
.bonus-display {
  text-align: center;
  min-width: 100px;
}
.bonus-label {
  display: block;
  font-size: 0.75rem;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.bonus-value {
  font-size: 2rem;
  font-weight: 700;
  color: var(--primary);
}
.step-btn {
  width: 44px;
  height: 44px;
  padding: 0;
  font-size: 1.4rem;
  display: flex;
  align-items: center;
  justify-content: center;
}
.cost-info {
  background: var(--bg-elevated);
  border-radius: 10px;
  padding: 1rem;
}
.cost-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.3rem 0;
  font-size: 0.9rem;
  color: var(--text-muted);
}
.cost-value {
  color: var(--text);
  font-weight: 600;
}
.insufficient {
  color: var(--error) !important;
  font-weight: 600;
}
.actions {
  display: flex;
  gap: 0.75rem;
}
.actions button {
  flex: 1;
  padding: 0.7em;
}
</style>
