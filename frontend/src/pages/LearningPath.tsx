import { Link } from 'react-router-dom'
import { useLearningPath } from '../api/hooks'
import type { StageState } from '../api/types'
import { Icon } from '../components/Icon'
import { QueryState } from '../components/QueryState'
import { PageHeader, ProgressBar, Ring } from '../components/ui'
import { faNumber, faPercent, termLabel } from '../lib/format'

const STATE_LABEL: Record<StageState, string> = { passed: 'گذرانده', current: 'سال جاری', upcoming: 'پیش رو' }

export default function LearningPath() {
  const query = useLearningPath()
  return (
    <>
      <PageHeader eyebrow="نقشه‌ی تحصیلی" title="مسیر یادگیری" lead="شش سال، از جونیور تا سینیور پلاس. هر سال با گذراندن درس‌ها و تحویل پروژه‌ی پایانی به سال بعد می‌روی." />
      <QueryState query={query}>
        {(p) => (
          <>
            <div className="card path-summary">
              <Ring value={p.percent} size={104} />
              <div className="stack" style={{ gap: 4 }}>
                <strong className="serif" style={{ fontSize: '1.2rem' }}>
                  {p.graduated ? 'فارغ‌التحصیل' : `سال ${faNumber(p.currentYear)} از ۶`}
                </strong>
                <span className="text-2 small">{termLabel(p.currentTerm)}</span>
                <span className="small">
                  {faNumber(p.creditsPassed)} واحد گذرانده · {faNumber(p.totalCredits - p.creditsPassed)} واحد تا فارغ‌التحصیلی
                </span>
              </div>
            </div>
            <ol className="path-timeline">
              {p.stages.map((s) => (
                <li key={s.year} className={`stage ${s.state}`}>
                  <div className="stage-marker">
                    <span>{s.state === 'passed' ? <Icon name="check" /> : faNumber(s.year)}</span>
                  </div>
                  <div className="card stage-card">
                    <div className="row between wrap">
                      <div className="row wrap">
                        <h2 className="serif stage-title">{s.nameFa}</h2>
                        <span className="badge ltr">{s.nameEn}</span>
                      </div>
                      <span className={`badge ${s.state === 'passed' ? 'done' : s.state === 'current' ? 'enrolled' : 'locked'}`}>
                        {STATE_LABEL[s.state]}
                      </span>
                    </div>
                    <p className="text-2" style={{ margin: '6px 0 12px' }}>
                      {s.yearBlurb}
                    </p>
                    <p className="small muted">{s.description}</p>
                    <dl className="stage-facts">
                      <div>
                        <dt>سال تحصیلی</dt>
                        <dd>{faNumber(s.year)}</dd>
                      </div>
                      <div>
                        <dt>تعداد واحد</dt>
                        <dd>{faNumber(s.requiredCredits)}</dd>
                      </div>
                      <div>
                        <dt>تعداد درس</dt>
                        <dd>{s.courseCountLabel}</dd>
                      </div>
                      <div>
                        <dt>پروژه</dt>
                        <dd>{s.project ? <Link to={`/courses/${s.project.slug}`}>{s.project.title.replace(/^پروژه[^:]*:\s*/, '')}</Link> : '—'}</dd>
                      </div>
                    </dl>
                    <div className="row between tiny muted" style={{ marginTop: 12 }}>
                      <span>
                        {faNumber(s.creditsPassed)} از {faNumber(s.requiredCredits)} واحد
                      </span>
                      <span>{faPercent(s.percent)}</span>
                    </div>
                    <ProgressBar value={s.percent} label={`پیشرفت ${s.nameFa}`} />
                    <div className="row wrap" style={{ marginTop: 12, gap: 6 }}>
                      {s.courses.map((c) => (
                        <Link key={c.slug} to={`/courses/${c.slug}`} className={`badge ${c.done ? 'done' : ''}`}>
                          {c.done && <Icon name="check" />} {c.code}
                        </Link>
                      ))}
                    </div>
                  </div>
                </li>
              ))}
              <li className={`stage graduation ${p.graduated ? 'passed' : 'upcoming'}`}>
                <div className="stage-marker">
                  <span>
                    <Icon name="cap" />
                  </span>
                </div>
                <div className="card stage-card brass-edge">
                  <h2 className="serif stage-title">فارغ‌التحصیلی</h2>
                  <p className="text-2 small">دفاع از پروژه‌ی پایانی و دریافت گواهی فارغ‌التحصیلی نهایی کد آکادمی.</p>
                </div>
              </li>
            </ol>
          </>
        )}
      </QueryState>
    </>
  )
}
