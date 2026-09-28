import { Link } from 'react-router-dom'
import type { CourseDetail } from '../api/types'
import { Icon } from '../components/Icon'
import { Grade, ProgressBar, StatusBadge } from '../components/ui'
import { faNumber } from '../lib/format'

export function CourseHeader({ c }: { c: CourseDetail }) {
  return (
    <div className="row wrap" style={{ gap: 10, flex: 1, minWidth: 0 }}>
      <span className="code-tag">{c.code}</span>
      <strong className="course-name">{c.title}</strong>
      <span className="row wrap" style={{ gap: 6, marginInlineStart: 'auto' }}>
        {c.kind === 'project' && <span className="badge brass">پروژه</span>}
        <span className="badge">{faNumber(c.credits)} واحد</span>
        <span className="badge">{c.levelName}</span>
        {c.grade != null && <Grade value={c.grade} />}
        <StatusBadge status={c.status} grade={c.grade} />
      </span>
    </div>
  )
}

export function CourseBody({ c }: { c: CourseDetail }) {
  return (
    <div className="stack" style={{ paddingTop: 12 }}>
      <p className="text-2 small">{c.summary}</p>
      <div className="row wrap small" style={{ gap: 16 }}>
        <span>
          <span className="muted">پیش‌نیاز: </span>
          {c.prerequisites.length === 0
            ? 'ندارد'
            : c.prerequisites.map((p, i) => (
                <span key={p.slug}>
                  {i > 0 && '، '}
                  <Link to={`/courses/${p.slug}`} style={{ color: p.done ? 'var(--mint)' : 'var(--text-2)' }}>
                    {p.code} {p.done && '✓'}
                  </Link>
                </span>
              ))}
        </span>
        <span>
          <span className="muted">مهارت‌ها: </span>
          {c.skills.join('، ')}
        </span>
      </div>
      <div className="row between tiny muted">
        <span>
          {faNumber(c.lessonsDone)} از {faNumber(c.lessonCount)} Lesson
        </span>
      </div>
      <ProgressBar value={c.percent} thin label="پیشرفت" />
      <ol className="lesson-list">
        {c.lessons.map((l) => (
          <li key={l.id}>
            <Link to={`/lessons/${l.id}`} className={l.completed ? 'done' : ''}>
              <span className="step-dot">{l.completed ? <Icon name="check" /> : faNumber(l.order)}</span>
              <span style={{ flex: 1 }}>{l.title}</span>
              <span className="tiny muted">{faNumber(l.estimatedMinutes)} دقیقه</span>
            </Link>
          </li>
        ))}
      </ol>
      <Link to={`/courses/${c.slug}`} className="card-link">
        صفحه‌ی کامل درس ←
      </Link>
    </div>
  )
}
