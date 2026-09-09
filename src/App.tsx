import { useState } from 'react'
import { Pause, Play, ShieldCheck } from '@phosphor-icons/react'
import { DateRangePicker } from './components/DateRangePicker'
import { VersionBadge } from './components/VersionBadge'
import { Overview } from './features/dashboard/Overview'
import { useAppSnapshot } from './features/dashboard/useAppSnapshot'
import { HistoryPage } from './features/history/HistoryPage'
import { initialHistory } from './features/history/history'
import { PAGES, usePage } from './features/navigation/usePage'
import { RankingPage } from './features/ranking/RankingPage'
import { initialRanking } from './features/ranking/ranking'
import type { DateSelection, HistoryTarget } from './lib/contracts'
import { nativeBridgeAvailable, setCollectionEnabled } from './lib/tauri'
import './App.css'

const browserDemo = !nativeBridgeAvailable()
const pageLabels = { overview: '概览', ranking: '排行', history: '记录' }
const pageTitles = { overview: '你的键盘热力', ranking: '每一个键，都有迹可循', history: '回看你的使用节奏' }
const pageCopy = {
  overview: '只查看聚合后的按键习惯。按得越多，颜色越深；不保存输入内容、顺序或正在使用的软件。',
  ranking: '完整查看 104 个物理按键与 85 个内置快捷键，找到常用键，也看见被忽略的键。',
  history: '从每日总量到小时明细，回看习惯如何变化。记录来自本机已保存的统计，而非输入内容。',
}

function App() {
  const { page, navigate } = usePage()
  const [selection, setSelection] = useState<DateSelection>({ range: 'today' })
  const [selectedKeyId, setSelectedKeyId] = useState<number | null>(null)
  const [ranking, setRanking] = useState(initialRanking)
  const [history, setHistory] = useState(initialHistory)
  const [revision, setRevision] = useState(0)
  const [busy, setBusy] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const { snapshot: appSnapshot, error, isLoading, isStale, refreshNow } = useAppSnapshot(selection, page)
  const snapshot = appSnapshot?.dashboard
  const collectionHealth = appSnapshot?.collection.overall
  const collectionEnabled = appSnapshot?.collection.enabled ?? false
  const collectionAction = collectionHealth === 'unavailable' ? 'retry' : collectionEnabled ? 'pause' : 'resume'
  const refreshAll = () => { refreshNow(); setRevision((value) => value + 1) }
  const changeSelection = (next: DateSelection) => {
    setSelection(next)
    setRanking((old) => ({ ...old, page: 0 }))
    setHistory((old) => ({ ...old, page: 0, expanded: null }))
  }
  const viewHistory = (target: HistoryTarget) => {
    setHistory((old) => ({ ...old, target, page: 0, expanded: null, metric: target.kind }))
    navigate('history')
  }
  const toggleCollection = async () => {
    if (!appSnapshot) return
    setBusy(true)
    try {
      await setCollectionEnabled(collectionAction === 'retry' ? true : !collectionEnabled)
      setActionError(null)
      refreshAll()
    } catch {
      setActionError(collectionAction === 'pause' ? '未能暂停统计，请重试。' : '未能恢复统计，请稍后再试。')
    } finally { setBusy(false) }
  }

  return <div className="app-shell">
    <header className="topbar">
      <div className="brand-group"><a className="brand" href="#overview" aria-label="键力，返回概览"><span className="brand__mark" aria-hidden="true"><i /><i /><i /></span><span>键力</span></a><VersionBadge /></div>
      <nav className="topbar__nav" aria-label="主导航">{PAGES.map((id) => <a key={id} href={`#${id}`} className={page === id ? 'is-active' : ''} aria-current={page === id ? 'page' : undefined}>{pageLabels[id]}</a>)}</nav>
      <div className="topbar__actions"><span className={`source-badge${browserDemo ? ' source-badge--demo' : ''}`}><i aria-hidden="true" />{browserDemo ? '浏览器演示' : '仅本机'}</span>
        <button type="button" className={`pause-button${collectionEnabled ? '' : ' pause-button--resume'}`} onClick={() => void toggleCollection()} disabled={busy || !appSnapshot} aria-busy={busy}>
          {collectionEnabled ? <Pause size={15} weight="fill" aria-hidden="true" /> : <Play size={15} weight="fill" aria-hidden="true" />}
          {busy ? '处理中' : !appSnapshot ? '状态待读取' : collectionAction === 'retry' ? '尝试恢复' : collectionAction === 'pause' ? '暂停统计' : '继续统计'}
        </button>
      </div>
    </header>
    <main className="dashboard">
      <section className="page-intro" aria-labelledby="page-title"><div><p className="eyebrow">键盘热力 / {pageLabels[page]}</p><h1 id="page-title">{pageTitles[page]}</h1><p className="page-intro__copy">{pageCopy[page]}</p></div></section>
      <div className="shared-controls"><DateRangePicker selection={selection} onChange={changeSelection} /><button type="button" className="subtle-button refresh-button" onClick={refreshAll}>刷新数据</button></div>
      <div className="range-summary" aria-live="polite"><span>{snapshot ? `${snapshot.rangeStartDate} — ${snapshot.rangeEndDate} · 本机日期（含首尾两天）` : isLoading ? '正在读取所选范围…' : '所选范围暂时无法读取'}</span><span>{appSnapshot ? `${isStale ? '可能已过期 · ' : ''}更新于 ${new Date(appSnapshot.generatedAt * 1000).toLocaleTimeString('zh-CN')}` : ''}</span></div>
      {browserDemo ? <aside className="demo-notice" aria-label="浏览器演示说明"><ShieldCheck size={18} weight="fill" aria-hidden="true" /><span>浏览器演示：使用合成样例，包含无记录日期，不会读取实体键盘。</span></aside> : null}
      {actionError ? <p className="error-notice" role="alert">{actionError}</p> : null}
      {error ? <p className="error-notice" role="alert">{error}<button type="button" className="text-button" onClick={refreshAll}>重试</button></p> : null}
      {collectionHealth === 'degraded' || collectionHealth === 'unavailable' ? <p className="error-notice" role="status">{collectionHealth === 'degraded' ? '部分采集监听器异常，请在概览查看具体状态。' : '本机采集暂不可用；历史聚合仍保留，可尝试恢复。'}</p> : null}
      {appSnapshot && appSnapshot.storage.status !== 'healthy' ? <p className="error-notice" role="alert">本机数据暂时无法保存；历史计数可能不完整，恢复后会自动重试未写入的聚合。</p> : null}
      {!snapshot || !appSnapshot ? <section className="loading-panel panel" aria-busy={isLoading} aria-live="polite"><div><p className="eyebrow">本机数据</p><h2>{isLoading ? '正在读取统计' : '暂时无法读取统计'}</h2><p>{isLoading ? '正在确认采集器与本机聚合库状态。' : '请点击重试，不会用演示计数替代本机记录。'}</p></div></section>
        : page === 'overview' ? <Overview appSnapshot={appSnapshot} selectedKeyId={selectedKeyId} setSelectedKeyId={setSelectedKeyId} busy={busy} toggleCollection={toggleCollection} browserDemo={browserDemo} isStale={isStale} onRanking={() => navigate('ranking')} />
        : page === 'ranking' ? <RankingPage snapshot={snapshot} state={ranking} onChange={setRanking} onHistory={viewHistory} />
        : <HistoryPage selection={selection} state={history} onChange={setHistory} revision={revision} onRanking={() => navigate('ranking')} />}
    </main>
  </div>
}

export default App
