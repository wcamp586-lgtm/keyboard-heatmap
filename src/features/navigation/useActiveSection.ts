import { useEffect, useState } from 'react'

export const NAVIGATION_SECTION_IDS = ['overview', 'ranking', 'privacy'] as const

export type NavigationSectionId = typeof NAVIGATION_SECTION_IDS[number]

type VisibleSection = {
  id: NavigationSectionId
  top: number
}

export const nearestVisibleSection = (
  sections: readonly VisibleSection[],
  headerOffset = 68,
): NavigationSectionId | null => sections.reduce<VisibleSection | null>((nearest, section) => {
  if (!nearest) return section
  return Math.abs(section.top - headerOffset) < Math.abs(nearest.top - headerOffset)
    ? section
    : nearest
}, null)?.id ?? null

export function useActiveSection(
  ready: boolean,
  initialSection: NavigationSectionId = 'overview',
) {
  const [activeSection, setActiveSection] = useState<NavigationSectionId>(initialSection)

  useEffect(() => {
    if (!ready || !('IntersectionObserver' in window)) return undefined

    const visibleSections = new Map<NavigationSectionId, number>()
    const observer = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        const id = entry.target.id as NavigationSectionId
        if (entry.isIntersecting) {
          visibleSections.set(id, entry.boundingClientRect.top)
        } else {
          visibleSections.delete(id)
        }
      }

      const nearest = nearestVisibleSection(
        [...visibleSections].map(([id, top]) => ({ id, top })),
      )
      if (nearest) setActiveSection(nearest)
    }, {
      rootMargin: '-68px 0px -10% 0px',
      threshold: [0, 0.01, 0.5],
    })

    for (const id of NAVIGATION_SECTION_IDS) {
      const section = document.getElementById(id)
      if (section) observer.observe(section)
    }

    return () => observer.disconnect()
  }, [ready])

  return { activeSection, setActiveSection }
}
