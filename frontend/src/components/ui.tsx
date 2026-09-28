import type { ReactNode } from 'react'
import type { EnrollmentStatus, NodeState } from '../api/types'
import { faGrade, faPercent } from '../lib/format'
import { Icon } from './Icon'

export function PageHeader({ eyebrow, title, lead, actions }: { eyebrow?: ReactNode; title: string; lead?: ReactNode; actions?: ReactNode }) {
  return (
    <header className="page-head">
      <div>
        {eyebrow && <div className="eyebrow">{eyebrow}</div>}
        <h1>{title}</h1>
        {lead && <p className="lead">{lead}</p>}
      </div>
      {actions && <div className="row wrap">{actions}</div>}
    </header>
  )
}

export function ProgressBar({ value, tone = 'mint', thin, label }: { value: number; tone?: 'mint' | 'brass'; thin?: boolean; label?: string }) {
  const v = Math.max(0, Math.min(100, value))
  return (
    <div
      className={`bar ${tone === 'brass' ? 'brass' : ''} ${thin ? 'thin' : ''}`}
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(v)}
      aria-valuetext={faPercent(v)}
      aria-label={label}
    >
      <span style={{ width: `${v}%` }} />
    </div>
  )
}

export function Ring({ value, size = 112, stroke = 9, tone = 'mint', children }: { value: number; size?: number; stroke?: number; tone?: 'mint' | 'brass'; children?: ReactNode }) {
  const r = (size - stroke) / 2
  const c = 2 * Math.PI * r
  const v = Math.max(0, Math.min(100, value))
  return (
    <div className="ring" style={{ width: size, height: size }} role="img" aria-label={faPercent(v)}>
      <svg width={size} height={size}>
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="var(--panel-2)" strokeWidth={stroke} />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke={tone === 'brass' ? 'var(--brass)' : 'var(--mint)'}
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={c}
          strokeDashoffset={c * (1 - v / 100)}
          style={{ transition: 'stroke-dashoffset .8s ease' }}
        />
      </svg>
      <div className="ring-label">{children ?? <span className="ring-value">{faPercent(v)}</span>}</div>
    </div>
  )
}

const STATUS: Record<EnrollmentStatus, { label: string; cls: string; icon: string }> = {
  done: { label: 'گذرانده', cls: 'done', icon: 'check' },
  in_progress: { label: 'در حال گذراندن', cls: 'progress', icon: 'play' },
  enrolled: { label: 'ثبت‌نام‌شده', cls: 'enrolled', icon: 'book' },
  locked: { label: 'قفل', cls: 'locked', icon: 'lock' },
}

export function StatusBadge({ status, grade }: { status: EnrollmentStatus; grade?: number | null }) {
  if (status === 'done' && grade != null && grade < 14) {
    return (
      <span className="badge fail">
        <Icon name="x" /> مردود
      </span>
    )
  }
  const s = STATUS[status]
  return (
    <span className={`badge ${s.cls}`}>
      <Icon name={s.icon} /> {s.label}
    </span>
  )
}

const NODE: Record<NodeState, { label: string; cls: string; icon: string }> = {
  done: { label: 'انجام‌شده', cls: 'done', icon: 'check' },
  in_progress: { label: 'در حال انجام', cls: 'progress', icon: 'play' },
  locked: { label: 'قفل', cls: 'locked', icon: 'lock' },
}

export function NodeBadge({ state }: { state: NodeState }) {
  const s = NODE[state]
  return (
    <span className={`badge ${s.cls}`}>
      <Icon name={s.icon} /> {s.label}
    </span>
  )
}

export function Grade({ value }: { value: number | null | undefined }) {
  if (value == null) return <span className="muted">—</span>
  return <strong style={{ color: value >= 14 ? 'var(--mint)' : 'var(--danger)' }}>{faGrade(value)}</strong>
}

export function Stat({ label, value, unit, icon, tone }: { label: string; value: ReactNode; unit?: string; icon?: string; tone?: 'brass' | 'mint' }) {
  return (
    <div className="stat">
      <span className="label">
        {icon && <Icon name={icon} />} {label}
      </span>
      <span className="value" style={tone ? { color: tone === 'brass' ? 'var(--brass-light)' : 'var(--mint)' } : undefined}>
        {value}
        {unit && <small>{unit}</small>}
      </span>
    </div>
  )
}

export function Card({ title, icon, action, children, className = '' }: { title?: ReactNode; icon?: string; action?: ReactNode; children: ReactNode; className?: string }) {
  return (
    <section className={`card ${className}`}>
      {(title || action) && (
        <div className="card-title">
          {title && (
            <h3>
              {icon && <Icon name={icon} />}
              {title}
            </h3>
          )}
          {action}
        </div>
      )}
      {children}
    </section>
  )
}
