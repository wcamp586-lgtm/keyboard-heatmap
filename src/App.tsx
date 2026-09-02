import { useEffect, useMemo, useState } from 'react'
import {
  ChartBar,
  Command,
  Keyboard,
  Pause,
  Play,
  ShieldCheck,
  TrendUp,
} from '@phosphor-icons/react'
import { BarChart } from './components/BarChart'
import { HourlyChart } from './components/HourlyChart'
import { KeyboardHeatmap } from './components/KeyboardHeatmap'
import { MetricCard } from './components/MetricCard'
import { ShortcutChart } from './components/ShortcutChart'
import type { DashboardSnapshot, PrivacyStatus, RangeKind } from './lib/contracts'
import { RANGE_OPTIONS } from './lib/contracts'
import { getKeyLabel } from './lib/keyboard-layout'
import { getMockDashboard } from './lib/mock-data'
import {
  getDashboard,
  getLiveCounts,
  getPrivacyStatus,
  nativeBridgeAvailable,
  setCollectionEnabled,
} from './lib/tauri'
import './App.css'

const numberFormatter = new Intl.NumberFormat('zh-CN')
const browserDemo = !nativeBridgeAvailable()

const rangeDescriptions: Record<RangeKind, string> = {
  today: '从今天 00:00 开始的本机聚合',
  week: '本周从周一 00:00 开始的本机聚合',
  month: '本月从 1 日 00:00 开始的本机聚合',
}

const defaultPrivacyStatus: PrivacyStatus = {
  isCollecting: true,
  collectionEnabled: true,
  collectionHealth: 'collecting',
  storage: '浏览器演示数据（不读取实体键盘）',
  capturesText: false,
}

const collectionStatusLabels = {
  collecting: '正在统计',
  paused: '已暂停',
  unavailable: '采集不可用',
} as const

function App() {
  const [range, setRange] = useState<RangeKind>('today')
  const [snapshot, setSnapshot] = useState<DashboardSnapshot>(() => getMockDashboard('today'))
  const [privacyStatus, setPrivacyStatus] = useState<PrivacyStatus>(defaultPrivacyStatus)
  const [selectedKeyId, setSelectedKeyId] = useState<number | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let active = true

    const refresh = async () => {
      try {
        const [nextSnapshot, liveCounts] = await Promise.all([getDashboard(range), getLiveCounts()])
        if (!active) return

        const collectionHealth = !nextSnapshot.collectionEnabled
          ? 'paused'
          : nextSnapshot.collectionHealth === 'unavailable' || liveCounts.collectionHealth === 'unavailable'
            ? 'unavailable'
            : 'collecting'
        const isCollecting = collectionHealth === 'collecting'

        setSnapshot({ ...nextSnapshot, isCollecting, collectionHealth })
        setPrivacyStatus((current) => ({
          ...current,
          isCollecting,
          collectionEnabled: nextSnapshot.collectionEnabled,
          collectionHealth,
        }))
        setError(null)
      } catch {
        if (active) setError('暂时无法读取本机聚合数据；界面会在下一次刷新时重试。')
      }
    }

    void refresh()
    if (!browserDemo) {
      const intervalId = window.setInterval(() => void refresh(), 2_000)
      return () => {
        active = false
        window.clearInterval(intervalId)
      }
    }

    return () => {
      active = false
    }
  }, [range])

  useEffect(() => {
    let active = true
    void getPrivacyStatus()
      .then((status) => {
        if (active) setPrivacyStatus(status)
      })
      .catch(() => undefined)

    return () => {
      active = false
    }
  }, [])

  const selectedKey = useMemo(() => {
    if (selectedKeyId === null) return null
    const count = snapshot.heatmap.find(({ keyId }) => keyId === selectedKeyId)?.pressCount ?? 0
    return { label: getKeyLabel(selectedKeyId), count }
  }, [selectedKeyId, snapshot.heatmap])

  const topKeyLabel = snapshot.topKey ? getKeyLabel(snapshot.topKey.keyId) : '—'
  const topKeyCount = snapshot.topKey ? numberFormatter.format(snapshot.topKey.pressCount) : '0'
  const collectionEnabled = snapshot.collectionEnabled
  const collectionHealth = snapshot.collectionHealth
  const collectionStatusLabel = collectionStatusLabels[collectionHealth]
  const isCollecting = collectionHealth === 'collecting'
  const collectionAction = collectionHealth === 'unavailable'
    ? 'retry'
    : collectionEnabled
      ? 'pause'
      : 'resume'

  const toggleCollection = async () => {
    setBusy(true)
    try {
      const status = await setCollectionEnabled(collectionAction === 'retry' ? true : !collectionEnabled)
      setPrivacyStatus(status)
      setSnapshot((current) => ({
        ...current,
        isCollecting: status.isCollecting,
        collectionEnabled: status.collectionEnabled,
        collectionHealth: status.collectionHealth,
      }))
      setError(null)
    } catch {
      setError('暂停状态没有更新，请稍后再试。')
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="app-shell">
      <header className="topbar">
        <a className="brand" href="#overview" aria-label="键力，返回概览">
          <span className="brand__mark" aria-hidden="true">
            <i />
            <i />
            <i />
          </span>
          <span>键力</span>
        </a>

        <nav className="topbar__nav" aria-label="主导航">
          <a href="#overview">概览</a>
          <a href="#ranking">排行</a>
          <a href="#privacy">隐私</a>
        </nav>

        <div className="topbar__actions">
          <span className={`source-badge${browserDemo ? ' source-badge--demo' : ''}`}>
            <i aria-hidden="true" />
            {browserDemo ? '浏览器演示' : '仅本机'}
          </span>
          <button
            type="button"
            className={`pause-button${isCollecting ? '' : ' pause-button--resume'}`}
            onClick={() => void toggleCollection()}
            disabled={busy}
          >
            {isCollecting ? <Pause size={15} weight="fill" aria-hidden="true" /> : <Play size={15} weight="fill" aria-hidden="true" />}
            {busy ? '处理中' : collectionAction === 'retry' ? '尝试恢复' : collectionAction === 'pause' ? '暂停统计' : '继续统计'}
          </button>
        </div>
      </header>

      <main className="dashboard">
        <section className="page-intro" aria-labelledby="page-title">
          <div>
            <p className="eyebrow">实时键盘热力图</p>
            <h1 id="page-title">你的键盘热力</h1>
            <p className="page-intro__copy">
              只查看聚合后的按键习惯。按得越多，颜色越深；不保存输入内容、顺序或正在使用的软件。
            </p>
          </div>

          <div className="range-picker" aria-label="选择统计范围">
            {RANGE_OPTIONS.map((option) => (
              <button
                type="button"
                key={option.value}
                className={range === option.value ? 'is-active' : ''}
                aria-pressed={range === option.value}
                onClick={() => setRange(option.value)}
              >
                {option.label}
              </button>
            ))}
          </div>
        </section>

        {browserDemo ? (
          <aside className="demo-notice" aria-label="浏览器演示说明">
            <ShieldCheck size={18} weight="fill" aria-hidden="true" />
            <span>浏览器演示模式：此页展示固定样例数据，不会读取实体键盘。</span>
          </aside>
        ) : null}

        {error ? <p className="error-notice" role="status">{error}</p> : null}
        {collectionHealth === 'unavailable' ? (
          <p className="error-notice" role="status">
            本机采集暂不可用；已保留历史聚合数据。可先暂停再继续，以重新启用统计。
          </p>
        ) : null}

        <section className="keyboard-panel panel" id="overview" aria-labelledby="heatmap-title">
          <div className="panel__heading">
            <div>
              <p className="eyebrow">{rangeDescriptions[range]}</p>
              <h2 id="heatmap-title">104 键实时热力图</h2>
            </div>
            <span className={`collection-state${collectionHealth === 'paused' ? ' collection-state--paused' : ''}${collectionHealth === 'unavailable' ? ' collection-state--unavailable' : ''}`}>
              <i aria-hidden="true" />
              {collectionStatusLabel}
            </span>
          </div>

          <KeyboardHeatmap
            counts={snapshot.heatmap}
            selectedKeyId={selectedKeyId}
            onSelect={setSelectedKeyId}
          />

          <div className="keyboard-panel__footer">
            <div className="heat-legend" aria-label="热度图例">
              <span>低</span>
              <i className="heat-legend__step heat-legend__step--0" />
              <i className="heat-legend__step heat-legend__step--1" />
              <i className="heat-legend__step heat-legend__step--2" />
              <i className="heat-legend__step heat-legend__step--3" />
              <i className="heat-legend__step heat-legend__step--4" />
              <span>高</span>
            </div>
            <p className="selected-key-readout">
              {selectedKey ? (
                <>
                  <kbd>{selectedKey.label}</kbd>
                  <span>{numberFormatter.format(selectedKey.count)} 次按下</span>
                </>
              ) : (
                '点击任意键帽查看次数'
              )}
            </p>
          </div>
        </section>

        <section className="metrics-grid" aria-label="概览统计">
          <MetricCard
            icon={<Keyboard size={18} weight="bold" />}
            label="当前范围按键"
            value={numberFormatter.format(snapshot.totalPresses)}
            helper={RANGE_OPTIONS.find((option) => option.value === range)?.label ?? '今天'}
            tone="rose"
          />
          <MetricCard
            icon={<ChartBar size={18} weight="bold" />}
            label="已使用按键"
            value={`${snapshot.activeKeyCount} / 104`}
            helper="至少按过一次的物理键"
          />
          <MetricCard
            icon={<TrendUp size={18} weight="bold" />}
            label="最常用按键"
            value={topKeyLabel}
            helper={`${topKeyCount} 次按下`}
          />
          <MetricCard
            icon={<Command size={18} weight="bold" />}
            label="已识别快捷键"
            value={numberFormatter.format(snapshot.totalShortcuts)}
            helper="仅内置组合，单独汇总"
          />
        </section>

        <section className="analytics-grid" aria-label="按键分析图表">
          <BarChart entries={snapshot.topKeys} />
          <HourlyChart entries={snapshot.hourly} />
          <ShortcutChart entries={snapshot.topShortcuts} totalShortcuts={snapshot.totalShortcuts} />
        </section>

        <section className="privacy-panel" id="privacy" aria-labelledby="privacy-title">
          <div className="privacy-panel__icon" aria-hidden="true">
            <ShieldCheck size={26} weight="fill" />
          </div>
          <div className="privacy-panel__body">
            <p className="eyebrow">本机隐私边界</p>
            <h2 id="privacy-title">只保存“时间桶 + 键位 / 内置快捷键 + 次数”</h2>
            <p>
              数据存放在{privacyStatus.storage}。快捷键仅汇总内置组合的次数；它不包含字符、按键顺序、剪贴板、窗口标题、应用名称、设备标识或跨设备同步数据。
            </p>
          </div>
          <button
            type="button"
            className="privacy-panel__action"
            onClick={() => void toggleCollection()}
            disabled={busy}
          >
            {collectionAction === 'retry' ? '尝试恢复统计' : collectionAction === 'pause' ? '暂停本机统计' : '继续本机统计'}
          </button>
        </section>
      </main>
    </div>
  )
}

export default App
