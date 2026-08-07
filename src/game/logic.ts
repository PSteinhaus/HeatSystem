import type { Outcome, RollResult, CriticalType } from '../types'

export function bonusCost(bonus: number): number {
  return bonus * bonus
}

export function calculateModifier(bonus: number, heat: number): number {
  return Math.min(bonus - heat, 3)
}

function categorizeTotal(total: number): Outcome {
  if (total <= 5) return 'failure'
  if (total <= 8) return 'mixed'
  return 'success'
}

export function calculateOutcome(d1: number, d2: number, bonus: number, heat: number): RollResult {
  const modifier = calculateModifier(bonus, heat)
  const total = d1 + d2 + modifier
  const isDoubles = d1 === d2

  let outcome: Outcome
  let critical: CriticalType = null

  if (isDoubles && d1 <= 2) {
    outcome = 'failure'
    critical = 'critical_failure'
  } else if (isDoubles && d1 <= 4) {
    if (total <= 5) {
      outcome = 'mixed'
      critical = 'upgrade'
    } else {
      outcome = categorizeTotal(total)
    }
  } else if (isDoubles && d1 >= 5) {
    outcome = 'success'
    critical = 'critical_success'
  } else {
    outcome = categorizeTotal(total)
  }

  return { d1, d2, bonus, heat, modifier, total, outcome, critical }
}

export function calculateHopeGained(outcome: Outcome, chainHeatSum: number): number {
  if (outcome === 'success') return 4 + 2 * chainHeatSum * chainHeatSum
  if (outcome === 'mixed') return 4 + chainHeatSum * chainHeatSum
  return 4
}

export const OUTCOME_LABELS: Record<Outcome, string> = {
  success: 'Erfolg',
  mixed: 'Zweischneidig',
  failure: 'Fehlschlag',
}

export const CRITICAL_LABELS: Record<string, string> = {
  critical_success: 'Kritischer Erfolg!',
  critical_failure: 'Kritischer Fehlschlag!',
  upgrade: 'Aufgewertet',
}
