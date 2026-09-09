import { useEffect, useState } from 'react'

export const PAGES = ['overview', 'ranking', 'history'] as const
export type Page = (typeof PAGES)[number]
export const pageFromHash = (hash: string): Page => PAGES.find((page) => hash === `#${page}`) ?? 'overview'

export function usePage() {
  const [page, setPage] = useState(() => pageFromHash(window.location.hash))
  useEffect(() => {
    const change = () => { setPage(pageFromHash(window.location.hash)); window.scrollTo(0, 0) }
    window.addEventListener('hashchange', change)
    return () => window.removeEventListener('hashchange', change)
  }, [])
  return { page, navigate: (next: Page) => { window.location.hash = next } }
}
