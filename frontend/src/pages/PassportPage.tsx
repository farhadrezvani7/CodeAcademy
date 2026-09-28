import { usePassport } from '../api/hooks'
import { BrandMark, Icon } from '../components/Icon'
import { QueryState } from '../components/QueryState'
import { PageHeader } from '../components/ui'
import { faGrade, faNumber, faPercent, jalali, termLabel } from '../lib/format'

export default function PassportPage() {
  const query = usePassport()
  return (
    <>
      <PageHeader eyebrow="سند رسمی" title="پاسپورت کد آکادمی" lead="شناسنامه‌ی تحصیلی تو: مهر سال‌های گذرانده‌شده، نوار مهارت‌ها و جایگاهت بین هم‌سطح‌ها." />
      <QueryState query={query}>
        {(p) => (
          <div className="passport">
            <div className="passport-page id-page">
              <div className="passport-head">
                <BrandMark className="passport-logo" />
                <div>
                  <div className="serif passport-org">کد آکادمی</div>
                  <div className="tiny passport-sub">PASSPORT · گذرنامه‌ی دانشجویی</div>
                </div>
              </div>
              <div className="passport-id">
                <div className="passport-photo" aria-hidden="true">
                  {p.name.charAt(0)}
                </div>
                <dl className="passport-fields">
                  <div>
                    <dt>نام</dt>
                    <dd className="serif">{p.name}</dd>
                  </div>
                  <div>
                    <dt>شماره‌ی دانشجویی</dt>
                    <dd className="ltr mono">{p.studentId}</dd>
                  </div>
                  <div>
                    <dt>تاریخ عضویت</dt>
                    <dd>{jalali(p.joinedAt)}</dd>
                  </div>
                  <div>
                    <dt>سطح فعلی</dt>
                    <dd>
                      {p.levelName} <span className="ltr tiny">({p.levelNameEn})</span>
                    </dd>
                  </div>
                  <div>
                    <dt>سال تحصیلی</dt>
                    <dd>
                      سال {faNumber(p.year)} · {termLabel(p.currentTerm)}
                    </dd>
                  </div>
                  <div>
                    <dt>وضعیت فارغ‌التحصیلی</dt>
                    <dd>{p.graduated ? `فارغ‌التحصیل (${p.graduatedAt ? jalali(p.graduatedAt) : ''})` : 'در حال تحصیل'}</dd>
                  </div>
                  <div>
                    <dt>معدل کل</dt>
                    <dd>{faGrade(p.gpa)}</dd>
                  </div>
                  <div>
                    <dt>رتبه بین هم‌سطح‌ها</dt>
                    <dd>
                      {faNumber(p.rank.rank)} از {faNumber(p.rank.of)} ({p.rank.levelName})
                    </dd>
                  </div>
                </dl>
              </div>
              <div className="mrz ltr" aria-hidden="true">
                {`P<CODEACADEMY<<${p.studentId.replace(/-/g, '<')}<<<<<<<<`}
                <br />
                {`${p.totalXp}XP<<Y${p.year}<<T${p.currentTerm}<<${p.streakDays}D<<<<<<<<<<<<`}
              </div>
            </div>

            <div className="passport-page stamps-page">
              <h3 className="serif">مهر سال‌های تحصیلی</h3>
              <div className="stamps">
                {p.stamps.map((s) => (
                  <div key={s.year} className={`stamp ${s.passed ? 'passed' : ''}`} style={{ rotate: `${((s.year * 37) % 15) - 7}deg` }}>
                    <span className="stamp-year">سال {faNumber(s.year)}</span>
                    <strong className="serif">{s.nameFa}</strong>
                    {s.passed ? (
                      <>
                        <span className="tiny">معدل {faGrade(s.gpa)}</span>
                        <span className="tiny">{s.passedAt && jalali(s.passedAt)}</span>
                      </>
                    ) : (
                      <span className="tiny">
                        <Icon name="lock" size={12} /> در انتظار
                      </span>
                    )}
                  </div>
                ))}
              </div>
              <h3 className="serif" style={{ marginTop: 20 }}>
                نوار مهارت‌ها
              </h3>
              <div className="stack">
                {p.skills.map((s) => (
                  <div key={s.departmentSlug} className={`accent-${s.accent}`}>
                    <div className="row between small">
                      <span>{s.nameFa}</span>
                      <span className="tiny">
                        {faNumber(s.creditsPassed)}/{faNumber(s.credits)} واحد · {faPercent(s.percent)}
                      </span>
                    </div>
                    <div className="skill-strip">
                      <span style={{ width: `${s.percent}%` }} />
                    </div>
                  </div>
                ))}
              </div>
              <div className="row wrap small" style={{ marginTop: 18, gap: 14 }}>
                <span>
                  <Icon name="trophy" size={15} /> {faNumber(p.achievementsUnlocked)}/{faNumber(p.achievementsTotal)} دستاورد
                </span>
                <span>
                  <Icon name="medal" size={15} /> {faNumber(p.certificates.filter((c) => c.issued).length)} گواهی‌نامه
                </span>
                <span>
                  <Icon name="star" size={15} /> {faNumber(p.totalXp)} XP
                </span>
              </div>
            </div>
          </div>
        )}
      </QueryState>
    </>
  )
}
