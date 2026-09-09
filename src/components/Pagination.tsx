export function Pagination({ page, totalPages, onChange, label }: { page: number; totalPages: number; onChange: (page: number) => void; label: string }) {
  return <nav className="pagination" aria-label={label}>
    <span>第 {page + 1} / {totalPages} 页</span>
    <button type="button" className="subtle-button" disabled={page === 0} onClick={() => onChange(page - 1)}>上一页</button>
    <button type="button" className="subtle-button" disabled={page + 1 >= totalPages} onClick={() => onChange(page + 1)}>下一页</button>
  </nav>
}
