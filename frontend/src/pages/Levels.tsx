import { Link } from 'react-router-dom'
import { useLevels } from '../api/hooks'
import { Icon } from '../components/Icon'
import { QueryState } from '../components/QueryState'
import { Card, PageHeader } from '../components/ui'
import { faNumber } from '../lib/format'

export default function Levels() {
  const query = useLevels()
  return (
    <>
      <PageHeader eyebrow="آیین‌نامه‌ی آموزشی" title="سطوح و قوانین ارتقا" lead="هر سطح یک سال تحصیلی است. برای ارتقا باید هر سه شرط زیر را در سال جاری برآورده کنی." />
      <QueryState query={query}>
        {(data) => (
          <>
            <section className="rules card brass-edge">
              <h2 className="serif" style={{ fontSize: '1.1rem', marginBottom: 12 }}>
                قوانین ارتقا (برای همه‌ی سطوح)
              </h2>
              <ol className="rules-list">
                {data.rules.map((r, i) => (
                  <li key={r.key}>
                    <span className="rule-no">{faNumber(i + 1)}</span>
                    <div>
                      <strong>{r.title}</strong>
                      <p className="small text-2">{r.description}</p>
                    </div>
                  </li>
                ))}
              </ol>
            </section>

            <div className="grid grid-2" style={{ marginTop: 20 }}>
              {data.levels.map((l) => (
                <Card key={l.year} className={`level-card ${l.state}`}>
                  <div className="row between">
                    <div className="row">
                      <span className="level-year">{faNumber(l.year)}</span>
                      <div>
                        <h2 className="serif" style={{ fontSize: '1.2rem' }}>
                          {l.nameFa}
                        </h2>
                        <span className="tiny muted ltr">{l.nameEn}</span>
                      </div>
                    </div>
                    {l.state === 'current' && <span className="badge enrolled">سطح فعلی تو</span>}
                    {l.state === 'passed' && (
                      <span className="badge done">
                        <Icon name="check" /> گذرانده
                      </span>
                    )}
                  </div>
                  <p className="text-2 small" style={{ margin: '10px 0' }}>
                    {l.yearBlurb}
                  </p>
                  <dl className="kv">
                    <dt>سال تحصیلی</dt>
                    <dd>{faNumber(l.year)}</dd>
                    <dt>واحد موردنیاز</dt>
                    <dd>{faNumber(l.requiredCredits)}</dd>
                    <dt>تعداد درس</dt>
                    <dd>{l.courseCountLabel}</dd>
                    <dt>پروژه‌ی پایانی</dt>
                    <dd>{l.project ? <Link to={`/courses/${l.project.slug}`}>{l.project.title}</Link> : '—'}</dd>
                  </dl>
                  {l.evaluation && (
                    <div className="level-eval">
                      <div className="tiny muted" style={{ marginBottom: 6 }}>
                        وضعیت شرایط ارتقا
                      </div>
                      {l.evaluation.rules.map((r) => (
                        <div key={r.key} className="row small" style={{ gap: 8 }}>
                          <span className={`rule-icon sm ${r.met ? 'met' : ''}`}>
                            <Icon name={r.met ? 'check' : 'clock'} />
                          </span>
                          <span>{r.title}</span>
                          {!r.met && r.blocking.length > 0 && <span className="tiny muted">({r.blocking.join('، ')})</span>}
                        </div>
                      ))}
                    </div>
                  )}
                </Card>
              ))}
            </div>
          </>
        )}
      </QueryState>
    </>
  )
}
