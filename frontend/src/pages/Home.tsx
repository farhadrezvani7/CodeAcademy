import { Link } from 'react-router-dom'
import { useOverview, useSession } from '../api/hooks'
import { Icon } from '../components/Icon'
import { QueryState, Skeleton } from '../components/QueryState'
import { faNumber } from '../lib/format'

export default function Home() {
  const overview = useOverview()
  const session = useSession()
  const signedIn = Boolean(session.data)
  const start = signedIn ? '/dashboard' : '/register'
  return (
    <div className="home">
      <section className="hero">
        <div className="hero-copy">
          <div className="eyebrow">دانشگاه آنلاین مهندسی نرم‌افزار</div>
          <h1>
            از اولین خط کد تا <span className="brass-text">سینیور پلاس</span>
            <br />
            در یک مسیر شش‌ساله‌ی دانشگاهی
          </h1>
          <p className="lead">
            کد آکادمی جایی است که مهندسی نرم‌افزار، Dart و Flutter را مثل یک دانشجو یاد می‌گیری: واحد می‌گذرانی، پروژه تحویل می‌دهی، نمره
            می‌گیری و سال‌به‌سال ارتقا پیدا می‌کنی تا فارغ‌التحصیل شوی.
          </p>
          <div className="row wrap" style={{ marginTop: 22 }}>
            <Link to={start} className="btn primary">
              {signedIn ? 'ادامه‌ی مسیر در داشبورد' : 'ثبت‌نام و شروع مسیر'} <Icon name="arrowLeft" />
            </Link>
            <Link to="/path" className="btn ghost">
              مشاهده‌ی مسیر شش‌ساله
            </Link>
          </div>
        </div>
        <div className="hero-seal" aria-hidden="true">
          <svg viewBox="0 0 200 200">
            <defs>
              <path id="seal-circle" d="M100 100 m-76 0 a76 76 0 1 1 152 0 a76 76 0 1 1 -152 0" />
            </defs>
            <circle cx="100" cy="100" r="94" fill="none" stroke="var(--brass)" strokeWidth="1.5" />
            <circle cx="100" cy="100" r="86" fill="none" stroke="var(--brass)" strokeWidth=".8" strokeDasharray="2 4" />
            <text fill="var(--brass-light)" fontSize="12.5" fontFamily="var(--font-serif)" letterSpacing="2">
              <textPath href="#seal-circle">کد آکادمی · مهندسی نرم‌افزار · Dart · Flutter · </textPath>
            </text>
            <g transform="translate(62 58) scale(1.2)">
              <path d="M32 10 8 22l24 12 24-12z" fill="#c9a227" />
              <path d="M16 28v12c0 5 7 10 16 10s16-5 16-10V28L32 36z" fill="#e6c667" opacity=".85" />
            </g>
          </svg>
        </div>
      </section>

      <QueryState query={overview} loading={<Skeleton height={90} />}>
        {(o) => (
          <>
            <section className="home-stats card">
              {[
                ['سال تحصیلی', o.stats.years],
                ['دانشکده', o.stats.departments],
                ['درس', o.stats.courses],
                ['پروژه', o.stats.projects],
                ['Lesson', o.stats.lessons],
                ['واحد', o.stats.credits],
                ['دانشجو', o.stats.students],
              ].map(([label, value]) => (
                <div key={label as string} className="home-stat">
                  <strong>{faNumber(value as number)}</strong>
                  <span>{label}</span>
                </div>
              ))}
            </section>

            <h2 className="section-title">فلسفه‌ی آموزشی</h2>
            <div className="grid grid-4">
              {o.highlights
                .filter((h) => h.section === 'principles')
                .map((p) => (
                <div key={p.title} className="card principle">
                  <Icon name={p.icon} className="principle-icon" />
                  <h3>{p.title}</h3>
                  <p className="text-2 small">{p.body}</p>
                </div>
              ))}
            </div>

            <h2 className="section-title">مسیر شش‌ساله</h2>
            <ol className="home-timeline">
              {o.levels.map((l) => (
                <li key={l.year}>
                  <span className="year-dot">{faNumber(l.year)}</span>
                  <div>
                    <div className="row wrap">
                      <strong className="serif">{l.nameFa}</strong>
                      <span className="badge brass ltr">{l.nameEn}</span>
                    </div>
                    <p className="text-2 small">{l.yearBlurb}</p>
                    <span className="tiny muted">
                      {faNumber(l.requiredCredits)} واحد · {l.courseCountLabel}
                    </span>
                  </div>
                </li>
              ))}
            </ol>

            <h2 className="section-title">دانشکده‌ها</h2>
            <div className="grid grid-4">
              {o.departments.map((d) => (
                <Link key={d.slug} to={`/departments/${d.slug}`} className={`card dept-card accent-${d.accent}`}>
                  <div className="dept-bar" />
                  <h3 className="serif">{d.nameFa}</h3>
                  <p className="text-2 small">{d.description}</p>
                  <span className="tiny muted">{faNumber(d.courseCount)} درس</span>
                </Link>
              ))}
            </div>

            <section className="cta card brass-edge">
              <div>
                <h2>{signedIn ? 'Lesson امروزت منتظر است' : 'آماده‌ی ثبت‌نام در سال اول هستی؟'}</h2>
                <p className="text-2">
                  {signedIn ? 'از داشبورد، درس فعلی و مأموریت امروزت را ببین.' : 'ثبت‌نام رایگان است؛ پرونده‌ی تحصیلی‌ات ساخته می‌شود و از سال اول شروع می‌کنی.'}
                </p>
              </div>
              <Link to={start} className="btn primary">
                {signedIn ? 'ورود به داشبورد' : 'ثبت‌نام'} <Icon name="arrowLeft" />
              </Link>
            </section>
          </>
        )}
      </QueryState>
    </div>
  )
}
