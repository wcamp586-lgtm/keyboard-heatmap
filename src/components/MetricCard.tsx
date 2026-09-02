import type { ReactNode } from 'react'

type MetricCardProps = {
  icon: ReactNode
  label: string
  value: string
  helper: string
  tone?: 'default' | 'rose'
}

export function MetricCard({ icon, label, value, helper, tone = 'default' }: MetricCardProps) {
  return (
    <article className={`metric-card metric-card--${tone}`}>
      <div className="metric-card__heading">
        <span className="metric-card__icon" aria-hidden="true">
          {icon}
        </span>
        <span>{label}</span>
      </div>
      <strong className="metric-card__value">{value}</strong>
      <p className="metric-card__helper">{helper}</p>
    </article>
  )
}
