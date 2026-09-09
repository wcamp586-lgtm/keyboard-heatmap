import { describe, expect, it } from 'vitest'
import { nearestVisibleSection } from './useActiveSection'

describe('navigation section selection', () => {
  it('picks the visible section nearest the sticky header', () => {
    expect(nearestVisibleSection([
      { id: 'overview', top: -180 },
      { id: 'ranking', top: 82 },
    ])).toBe('ranking')
  })

  it('keeps the current state unchanged when no section is visible', () => {
    expect(nearestVisibleSection([])).toBeNull()
  })
})
