<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'

const props = defineProps<{
  d1: number
  d2: number
  trigger: number
}>()

const faceTransforms: Record<number, string> = {
  1: 'rotateX(0deg) rotateY(0deg)',
  2: 'rotateX(0deg) rotateY(-90deg)',
  3: 'rotateX(-90deg) rotateY(0deg)',
  4: 'rotateX(90deg) rotateY(0deg)',
  5: 'rotateX(0deg) rotateY(90deg)',
  6: 'rotateX(0deg) rotateY(180deg)',
}

const rollCount1 = ref(0)
const rollCount2 = ref(0)
const transform1 = ref(faceTransforms[1])
const transform2 = ref(faceTransforms[1])
const rolling = ref(false)

function rollDie(target: number, die: 1 | 2): string {
  const spinsX = (Math.floor(Math.random() * 2) + 3) * 360
  const spinsY = (Math.floor(Math.random() * 2) + 3) * 360
  const dir = Math.random() > 0.5 ? 1 : -1
  const base = faceTransforms[target]
  // extract the numeric rotations from base and add spins
  const xMatch = base.match(/rotateX\((-?\d+)deg\)/)
  const yMatch = base.match(/rotateY\((-?\d+)deg\)/)
  const xBase = xMatch ? parseInt(xMatch[1]) : 0
  const yBase = yMatch ? parseInt(yMatch[1]) : 0
  const finalX = xBase + spinsX * dir
  const finalY = yBase + spinsY * dir
  if (die === 1) rollCount1.value++
  else rollCount2.value++
  return `rotateX(${finalX}deg) rotateY(${finalY}deg)`
}

watch(() => props.trigger, () => {
  if (props.trigger === 0) return
  rolling.value = true
  transform1.value = rollDie(props.d1, 1)
  transform2.value = rollDie(props.d2, 2)
  setTimeout(() => { rolling.value = false }, 1400)
}, { immediate: false })

onMounted(() => {
  if (props.trigger > 0) {
    rolling.value = true
    transform1.value = rollDie(props.d1, 1)
    transform2.value = rollDie(props.d2, 2)
    setTimeout(() => { rolling.value = false }, 1400)
  }
})

const dieFaces = [1, 2, 3, 4, 5, 6]

// pip positions for each face value (as grid positions 1-9)
const pipLayouts: Record<number, number[]> = {
  1: [5],
  2: [1, 9],
  3: [1, 5, 9],
  4: [1, 3, 7, 9],
  5: [1, 3, 5, 7, 9],
  6: [1, 3, 4, 6, 7, 9],
}
</script>

<template>
  <div class="dice-container" :class="{ rolling }">
    <div class="die-wrap">
      <div class="die" :style="{ transform: transform1 }">
        <div
          v-for="face in dieFaces"
          :key="face"
          class="die-face"
          :class="`face-${face}`"
        >
          <div class="pips">
            <span
              v-for="pos in 9"
              :key="pos"
              class="pip-slot"
            >
              <span v-if="pipLayouts[face].includes(pos)" class="pip"></span>
            </span>
          </div>
        </div>
      </div>
    </div>
    <div class="die-wrap">
      <div class="die" :style="{ transform: transform2 }">
        <div
          v-for="face in dieFaces"
          :key="face"
          class="die-face"
          :class="`face-${face}`"
        >
          <div class="pips">
            <span
              v-for="pos in 9"
              :key="pos"
              class="pip-slot"
            >
              <span v-if="pipLayouts[face].includes(pos)" class="pip"></span>
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dice-container {
  display: flex;
  gap: 2.5rem;
  justify-content: center;
  align-items: center;
  perspective: 600px;
  padding: 1rem;
}
.die-wrap {
  width: 64px;
  height: 64px;
  perspective: 400px;
}
.die {
  width: 64px;
  height: 64px;
  position: relative;
  transform-style: preserve-3d;
  transition: transform 1.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}
.die.rolling,
.dice-container.rolling .die {
  transition: transform 1.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.die-face {
  position: absolute;
  width: 64px;
  height: 64px;
  background: linear-gradient(135deg, #f5f0e0, #d8d0bc);
  border: 1px solid #b0a890;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: inset 0 0 12px rgba(0, 0, 0, 0.15);
}
.pips {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  grid-template-rows: repeat(3, 1fr);
  width: 80%;
  height: 80%;
  gap: 2px;
}
.pip-slot {
  display: flex;
  align-items: center;
  justify-content: center;
}
.pip {
  width: 10px;
  height: 10px;
  background: #1a1a2e;
  border-radius: 50%;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.4);
}
.face-1  { transform: translateZ(32px); }
.face-2  { transform: rotateY(90deg) translateZ(32px); }
.face-3  { transform: rotateX(90deg) translateZ(32px); }
.face-4  { transform: rotateX(-90deg) translateZ(32px); }
.face-5  { transform: rotateY(-90deg) translateZ(32px); }
.face-6  { transform: rotateY(180deg) translateZ(32px); }
</style>
