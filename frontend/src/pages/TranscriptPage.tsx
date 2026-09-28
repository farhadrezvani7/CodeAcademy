import { Link } from 'react-router-dom'
import { useTranscript } from '../api/hooks'
import { Icon } from '../components/Icon'
import { EmptyState, QueryState } from '../components/QueryState'
import { Grade, PageHeader, StatusBadge } from '../components/ui'
import { faGrade, faNumber, faPercent, jalali, termLabel } from '../lib/format'

export default function TranscriptPage() {
  const query = useTranscript()
  return (
    <>
      <PageHeader
        eyebrow="اداره‌ی آموزش"
        title="کارنامه‌ی تحصیلی"
        lead="ریز نمرات به تفکیک نیمسال. معدل به صورت میانگین وزنی محاسبه می‌شود: مجموع (نمره × واحد) تقسیم بر مجموع واحدها."
        actions={
          <button className="btn ghost" onClick={() => window.print()}>
            <Icon name="transcript" /> چاپ کارنامه
          </button>
        }
      />
      <QueryState query={query}>
        {(t) => (
          <div className="transcript">
            <div className="transcript-head">
              <div>
                <div className="serif transcript-org">کد آکادمی — اداره‌ی آموزش</div>
                <div className="tiny">کارنامه‌ی رسمی دانشجو · تاریخ صدور {jalali(t.issuedAt)}</div>
              </div>
              <dl className="transcript-id">
                <div>
                  <dt>نام دانشجو</dt>
                  <dd>{t.studentName}</dd>
                </div>
                <div>
                  <dt>شماره‌ی دانشجویی</dt>
                  <dd className="ltr">{t.studentId}</dd>
                </div>
                <div>
                  <dt>سطح</dt>
                  <dd>
                    {t.levelName} · {termLabel(t.currentTerm)}
                  </dd>
                </div>
              </dl>
            </div>

            <div className="transcript-summary">
              <div>
                <span>معدل کل</span>
                <strong>{faGrade(t.cumulativeGpa)}</strong>
              </div>
              <div>
                <span>واحد گذرانده</span>
                <strong>{faNumber(t.creditsPassed)}</strong>
              </div>
              <div>
                <span>واحد باقی‌مانده</span>
                <strong>{faNumber(t.creditsRemaining)}</strong>
              </div>
              <div>
                <span>کل واحدهای دوره</span>
                <strong>{faNumber(t.totalCredits)}</strong>
              </div>
            </div>

            {t.semesters.length === 0 && <EmptyState title="هنوز نمره‌ای ثبت نشده است" />}
            {t.semesters.map((s) => (
              <section key={s.number} className="semester">
                <div className="semester-head">
                  <strong>
                    ترم {faNumber(s.number)} — {s.title}
                  </strong>
                  <span className="small">
                    {faNumber(s.credits)} واحد · معدل ترم <b>{faGrade(s.gpa)}</b>
                  </span>
                </div>
                <div className="table-wrap">
                  <table className="table stack-mobile">
                    <thead>
                      <tr>
                        <th>کد درس</th>
                        <th>نام درس</th>
                        <th>واحد</th>
                        <th>نمره (از ۲۰)</th>
                        <th>وضعیت</th>
                      </tr>
                    </thead>
                    <tbody>
                      {s.courses.map((c) => (
                        <tr key={c.slug}>
                          <td data-label="کد درس">
                            <span className="code-tag">{c.code}</span>
                          </td>
                          <td className="wrap" data-label="نام درس">
                            <Link to={`/courses/${c.slug}`}>{c.title}</Link>
                          </td>
                          <td data-label="واحد">{faNumber(c.credits)}</td>
                          <td data-label="نمره">
                            <Grade value={c.grade} />
                          </td>
                          <td data-label="وضعیت">{c.passed ? <span className="badge done">قبول</span> : <span className="badge fail">مردود</span>}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </section>
            ))}

            {t.inProgress.length > 0 && (
              <section className="semester current">
                <div className="semester-head">
                  <strong>ترم جاری — در حال گذراندن</strong>
                  <span className="small">{faNumber(t.inProgress.reduce((a, c) => a + c.credits, 0))} واحد</span>
                </div>
                <div className="table-wrap">
                  <table className="table stack-mobile">
                    <tbody>
                      {t.inProgress.map((c) => (
                        <tr key={c.slug}>
                          <td data-label="کد درس">
                            <span className="code-tag">{c.code}</span>
                          </td>
                          <td className="wrap" data-label="نام درس">
                            <Link to={`/courses/${c.slug}`}>{c.title}</Link>
                          </td>
                          <td data-label="واحد">{faNumber(c.credits)}</td>
                          <td className="muted" data-label="پیشرفت">{faPercent(c.percent)}</td>
                          <td data-label="وضعیت">
                            <StatusBadge status={c.status} />
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </section>
            )}
            <div className="transcript-foot tiny">
              <span>حداقل نمره‌ی قبولی هر درس: ۱۴ از ۲۰</span>
              <span className="transcript-sign serif">مهر و امضای اداره‌ی آموزش کد آکادمی</span>
            </div>
          </div>
        )}
      </QueryState>
    </>
  )
}
