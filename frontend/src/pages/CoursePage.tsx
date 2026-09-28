import { Link, useParams } from 'react-router-dom'
import { useCourse } from '../api/hooks'
import { Icon } from '../components/Icon'
import { QueryState } from '../components/QueryState'
import { Card, Grade, PageHeader, ProgressBar, StatusBadge } from '../components/ui'
import { faNumber, faPercent } from '../lib/format'

export default function CoursePage() {
  const { slug = '' } = useParams()
  const query = useCourse(slug)
  return (
    <QueryState query={query}>
      {(c) => (
        <>
          <PageHeader
            eyebrow={`${c.departmentName} · سال ${faNumber(c.year)} · نیمسال ${c.termInYear === 1 ? 'اول' : 'دوم'}`}
            title={c.title}
            lead={c.summary}
            actions={
              <>
                <span className="code-tag" style={{ fontSize: '.95rem' }}>
                  {c.code}
                </span>
                <StatusBadge status={c.status} grade={c.grade} />
              </>
            }
          />
          {c.needsRetake && (
            <div className="notice danger" role="status" style={{ marginBottom: 18 }}>
              <Icon name="alert" />
              <span>
                نمره‌ی این درس کمتر از ۱۴ شد. Lessonها را دوباره مرور کن و بعد از انجام تمرین، خودارزیابی را به‌روز کن تا نمره‌ات اصلاح شود.
              </span>
            </div>
          )}
          <div className="grid grid-3">
            <Card className="span-2" title="Lessonهای درس" icon="book">
              <ol className="lesson-list big">
                {c.lessons.map((l) => (
                  <li key={l.id}>
                    <Link to={`/lessons/${l.id}`} className={l.completed ? 'done' : l.id === c.nextLessonId ? 'next' : ''}>
                      <span className="step-dot">{l.completed ? <Icon name="check" /> : faNumber(l.order)}</span>
                      <span style={{ flex: 1 }}>{l.title}</span>
                      <span className="tiny muted">{faNumber(l.estimatedMinutes)} دقیقه</span>
                    </Link>
                  </li>
                ))}
              </ol>
              {c.status === 'locked' ? (
                <p className="small muted" style={{ marginTop: 12 }}>
                  <Icon name="lock" size={15} /> این درس بعد از گذراندن پیش‌نیازها و رسیدن به سال {faNumber(c.year)} باز می‌شود.
                </p>
              ) : (
                c.nextLessonId && (
                  <Link to={`/lessons/${c.nextLessonId}`} className="btn mint" style={{ marginTop: 14 }}>
                    <Icon name="play" /> {c.needsRetake ? 'مرور دوباره' : c.lessonsDone === 0 ? 'شروع درس' : 'ادامه‌ی درس'}
                  </Link>
                )
              )}
            </Card>
            <Card title="مشخصات درس" icon="transcript">
              <dl className="kv">
                <dt>کد درس</dt>
                <dd>{c.code}</dd>
                <dt>واحد</dt>
                <dd>{faNumber(c.credits)}</dd>
                <dt>سطح</dt>
                <dd>{c.levelName}</dd>
                <dt>نوع</dt>
                <dd>{c.kind === 'project' ? 'پروژه' : 'درس نظری-عملی'}</dd>
                <dt>نمره</dt>
                <dd>
                  <Grade value={c.grade} />
                </dd>
              </dl>
              <div style={{ margin: '16px 0 6px' }} className="row between tiny muted">
                <span>پیشرفت</span>
                <span>{faPercent(c.percent)}</span>
              </div>
              <ProgressBar value={c.percent} label="پیشرفت درس" />
              <h4 style={{ margin: '18px 0 8px', fontSize: '.85rem' }}>پیش‌نیازها</h4>
              {c.prerequisites.length === 0 ? (
                <p className="small muted">ندارد</p>
              ) : (
                <ul className="list">
                  {c.prerequisites.map((p) => (
                    <li key={p.slug}>
                      <Icon name={p.done ? 'check' : 'lock'} size={16} />
                      <Link to={`/courses/${p.slug}`} className="small">
                        {p.code} — {p.title}
                      </Link>
                    </li>
                  ))}
                </ul>
              )}
              <h4 style={{ margin: '18px 0 8px', fontSize: '.85rem' }}>مهارت‌های این درس</h4>
              <div className="row wrap" style={{ gap: 6 }}>
                {c.skills.map((s) => (
                  <span key={s} className="skill-pill">
                    {s}
                  </span>
                ))}
              </div>
            </Card>
          </div>
          <p style={{ marginTop: 18 }}>
            <Link to={`/departments/${c.departmentSlug}`} className="card-link">
              بازگشت به دانشکده {c.departmentName} ←
            </Link>
          </p>
        </>
      )}
    </QueryState>
  )
}
