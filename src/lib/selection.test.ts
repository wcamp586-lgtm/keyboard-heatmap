import { describe, expect, it } from 'vitest'
import { nextSelectedKeyId } from './selection'

describe('keyboard heatmap selection', () => {
  it('selects a new key and clears the same key when clicked again', () => {
    expect(nextSelectedKeyId(null, 94)).toBe(94)
    expect(nextSelectedKeyId(94, 94)).toBeNull()
    expect(nextSelectedKeyId(94, 70)).toBe(70)
  })
})
