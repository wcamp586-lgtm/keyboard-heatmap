import { ChartBar, Command, Keyboard, ShieldCheck, TrendUp } from '@phosphor-icons/react'
import { BarChart } from '../../components/BarChart'
import { HourlyChart } from '../../components/HourlyChart'
import { KeyboardHeatmap } from '../../components/KeyboardHeatmap'
import { MetricCard } from '../../components/MetricCard'
import { ShortcutChart } from '../../components/ShortcutChart'
import { RANGE_OPTIONS, type AppSnapshot } from '../../lib/contracts'
import { formatNumber } from '../../lib/format'
import { getDetailedKeyLabel } from '../ranking/ranking'

export function Overview({ appSnapshot, selectedKeyId, setSelectedKeyId, busy, toggleCollection, browserDemo, isStale, onRanking }: {
  appSnapshot: AppSnapshot; selectedKeyId: number | null; setSelectedKeyId: (id: number | null) => void;
  busy: boolean; toggleCollection: () => Promise<void>; browserDemo: boolean; isStale: boolean; onRanking: () => void
}) {
  const snapshot = appSnapshot.dashboard
  const range = snapshot.range
  const isLoading = false
  const collectionHealth = appSnapshot.collection.overall
  const collectionAction = collectionHealth === 'unavailable' ? 'retry' : appSnapshot.collection.enabled ? 'pause' : 'resume'
  const collectionStatusLabel = { collecting: '正在统计', degraded: '部分功能异常', paused: '已暂停', unavailable: '采集不可用' }[collectionHealth]
  const selectedKey = selectedKeyId === null ? null : { label: getDetailedKeyLabel(selectedKeyId), count: snapshot.heatmap.find((key) => key.keyId === selectedKeyId)?.pressCount ?? 0 }
  const topKeyLabel = snapshot.topKey ? getDetailedKeyLabel(snapshot.topKey.keyId) : '—'
  const topKeyCount = formatNumber(snapshot.topKey?.pressCount ?? 0)
  const formatUpdateTime = (timestamp: number) => new Date(timestamp * 1000).toLocaleTimeString('zh-CN')
  return <>
        <section className="keyboard-panel panel" id="overview" aria-labelledby="heatmap-title">
          <div className="panel__heading">
            <div>
              <p className="eyebrow">{`${snapshot.rangeStartDate} — ${snapshot.rangeEndDate} · 本机聚合`}</p>
              <h2 id="heatmap-title">104 键实时热力图</h2>
            </div>
            <span className={`collection-state${collectionHealth === 'paused' ? ' collection-state--paused' : ''}${collectionHealth === 'degraded' || collectionHealth === 'unavailable' ? ' collection-state--unavailable' : ''}`}>
              <i aria-hidden="true" />
              {collectionStatusLabel}
            </span>
          </div>

          <div className="health-summary" aria-label="当前运行状态">
            <span>实体键：{appSnapshot.collection.physicalListener === 'active' ? '正常' : appSnapshot.collection.physicalListener === 'paused' ? '已暂停' : '不可用'}</span>
            <span>快捷键：{appSnapshot.collection.shortcutListener === 'active' ? '正常' : appSnapshot.collection.shortcutListener === 'paused' ? '已暂停' : '不可用'}</span>
            <span>本机保存：{appSnapshot.storage.status === 'healthy' ? '正常' : appSnapshot.storage.status === 'degraded' ? '正在重试' : '不可用'}</span>
            <span>{isStale ? '数据可能已过期' : `更新于 ${formatUpdateTime(appSnapshot.generatedAt)}`}</span>
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
                  <span>{formatNumber(selectedKey.count)} 次按下</span>
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
            value={formatNumber(snapshot.totalPresses)}
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
            value={formatNumber(snapshot.totalShortcuts)}
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
              数据存放在{browserDemo ? '浏览器演示内存' : '本机 SQLite 聚合库'}。快捷键仅汇总内置组合的次数；它不包含字符、按键顺序、剪贴板、窗口标题、应用名称、设备标识或跨设备同步数据。
            </p>
          </div>
          <button
            type="button"
            className="privacy-panel__action"
            onClick={() => void toggleCollection()}
            disabled={busy || isLoading}
            aria-busy={busy}
          >
            {collectionAction === 'retry' ? '尝试恢复统计' : collectionAction === 'pause' ? '暂停本机统计' : '继续本机统计'}
          </button>
        </section>

    <div className="overview-more"><button type="button" className="subtle-button" onClick={onRanking}>查看完整排行与单项历史 →</button></div>
  </>
}
