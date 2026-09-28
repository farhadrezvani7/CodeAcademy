import { Link } from 'react-router-dom'
import { useCheckIn, useDashboard, useLogStudy } from '../api/hooks'
import type { Dashboard as DashboardData } from '../api/types'
import { Icon } from '../components/Icon'
import { QueryState, Skeleton } from '../components/QueryState'
import { useToast } from '../components/Toast'
import { Card, Grade, PageHeader, ProgressBar, Ring, Stat, StatusBadge } from '../components/ui'
import { clockTime, faDuration, faGrade, faNumber, faPercent, jalali, jalaliDayMonth, termLabel } from '../lib/format'

export default function Dashboard() {
  const query = useDashboard()
  return (
    <QueryState query={query} loading={<DashboardSkeleton />}>
      {(d) => <DashboardView d={d} />}
    </QueryState>
  )
}

function DashboardSkeleton() {
  return (
    <div className="stack">
      <Skeleton height={80} />
      <div className="grid grid-3">
        <Skeleton height={220} />
        <Skeleton height={220} />
        <Skeleton height={220} />
      </div>
      <Skeleton height={180} />
    </div>
  )
}

function DashboardView({ d }: { d: DashboardData }) {
  const s = d.student
  const firstName = s.name.split(' ')[0]
  return (
    <>
      <PageHeader
        eyebrow={jalali(d.today)}
        title={`سلام ${firstName}، خوش برگشتی`}
        lead={s.levelBlurb}
        actions={
          <>
            <span className="badge brass">
              <Icon name="levels" /> سال {faNumber(s.year)} · {s.levelName}
            </span>
            <span className="badge">{termLabel(s.currentTerm)}</span>
          </>
        }
      />

      <div className="grid dash-top">
        <Card className="brass-edge standing">
          <div className="row" style={{ gap: 20, alignItems: 'center' }}>
            <Ring value={d.progress.overallPercent} size={124}>
              <span className="ring-value">{faPercent(d.progress.overallPercent)}</span>
              <span className="tiny muted">کل مسیر</span>
            </Ring>
            <div className="stack" style={{ gap: 8, flex: 1, minWidth: 0 }}>
              <div className="tiny muted">سطح فعلی</div>
              <div className="serif standing-level">{s.levelName}</div>
              <div className="small text-2">
                {faNumber(d.progress.creditsPassed)} از {faNumber(d.progress.totalCredits)} واحد گذرانده
              </div>
              <div>
                <div className="row between tiny muted">
                  <span>پیشرفت سال {faNumber(s.year)}</span>
                  <span>
                    {faNumber(d.progress.yearCreditsPassed)}/{faNumber(d.progress.yearCredits)} واحد
                  </span>
                </div>
                <ProgressBar value={d.progress.yearPercent} tone="brass" label="پیشرفت سال" />
              </div>
            </div>
          </div>
        </Card>
        <Card className="stats-tiles">
          <Stat label="امتیاز تجربه" icon="star" value={faNumber(s.totalXp)} unit="XP" tone="brass" />
          <Stat label="روزهای متوالی" icon="flame" value={faNumber(s.streakDays)} unit="روز" tone="mint" />
          <Stat label="معدل کل" icon="transcript" value={faGrade(s.gpa)} unit="از ۲۰" />
          <Link to="/leaderboard">
            <Stat label={`رتبه بین ${d.rank.levelName}‌ها`} icon="users" value={faNumber(d.rank.rank)} unit={`از ${faNumber(d.rank.of)}`} />
          </Link>
        </Card>
      </div>

      <div className="grid grid-3" style={{ marginTop: 18 }}>
        <CurrentCourse d={d} />
        <MissionCard d={d} />
      </div>

      <div className="grid grid-3" style={{ marginTop: 18 }}>
        <CheckIn d={d} />
        <AdvisorCard d={d} />
        <ProjectCard d={d} />
      </div>

      <div className="grid grid-2" style={{ marginTop: 18 }}>
        <PromotionCard d={d} />
        <Card title="وضعیت مهارت‌ها" icon="tree" action={<Link className="card-link" to="/skills">درخت مهارت</Link>}>
          <div className="stack">
            {d.skills.map((sk) => (
              <div key={sk.departmentSlug}>
                <div className="row between small">
                  <span>{sk.nameFa}</span>
                  <span className="muted tiny">
                    {faPercent(sk.percent)}
                    {sk.openCourses > 0 && ` · ${faNumber(sk.openCourses)} درس باز`}
                  </span>
                </div>
                <ProgressBar value={sk.percent} label={sk.nameFa} />
              </div>
            ))}
          </div>
        </Card>
      </div>

      <div className="grid grid-3" style={{ marginTop: 18 }}>
        <Card title="دستاوردهای اخیر" icon="trophy" action={<Link className="card-link" to="/achievements">همه</Link>}>
          {d.recentAchievements.length === 0 ? (
            <p className="muted small">هنوز دستاوردی باز نکرده‌ای؛ اولین Lesson را کامل کن.</p>
          ) : (
            <ul className="list">
              {d.recentAchievements.map((a) => (
                <li key={a.slug}>
                  <span className="medal-sm">
                    <Icon name={a.icon} />
                  </span>
                  <div style={{ minWidth: 0 }}>
                    <div className="small" style={{ fontWeight: 600 }}>
                      {a.title}
                    </div>
                    <div className="tiny muted">{a.unlockedAt && jalali(a.unlockedAt)}</div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Card>
        <Card title="فعالیت‌های اخیر" icon="clock" action={<Link className="card-link" to="/stats">آمار</Link>}>
          {d.recentActivity.length === 0 ? (
            <p className="muted small">هنوز فعالیتی ثبت نشده است.</p>
          ) : (
            <ul className="list">
              {d.recentActivity.map((a, i) => (
                <li key={i}>
                  <span className="dot" style={{ background: a.source === 'lesson' ? 'var(--mint)' : 'var(--brass)' }} />
                  <div style={{ minWidth: 0, flex: 1 }}>
                    <div className="small">{a.source === 'lesson' ? `تکمیل Lesson — ${a.courseTitle ?? a.topic}` : `مطالعه — ${a.courseTitle ?? a.topic}`}</div>
                    <div className="tiny muted">
                      {jalaliDayMonth(a.date)} · ساعت {clockTime(a.at)} · {faDuration(a.minutes)}
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Card>
        <Card title="اطلاعیه‌ها" icon="bell">
          {d.announcements.length === 0 ? (
            <p className="muted small">اطلاعیه‌ی تازه‌ای نیست.</p>
          ) : (
            <ul className="list">
              {d.announcements.map((a) => (
                <li key={a.id} style={{ alignItems: 'flex-start' }}>
                  <span className="dot" style={{ marginTop: 10, background: a.important ? 'var(--brass)' : 'var(--muted)' }} />
                  <div style={{ minWidth: 0 }}>
                    <div className="small" style={{ fontWeight: 600 }}>
                      {a.title} {a.important && <span className="badge brass">مهم</span>}
                    </div>
                    <p className="tiny text-2">{a.body}</p>
                    <div className="tiny muted">{jalali(a.publishedAt)}</div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Card>
      </div>
    </>
  )
}

function CurrentCourse({ d }: { d: DashboardData }) {
  const c = d.currentCourse
  if (!c) {
    return (
      <Card title="درس فعلی" icon="book" className="span-2">
        <p className="muted">درس بازی نداری. شرایط ارتقا را در صفحه‌ی سطوح ببین.</p>
      </Card>
    )
  }
  const next = c.lessons.find((l) => l.id === c.nextLessonId)
  return (
    <Card title="درس فعلی" icon="book" className="span-2 mint-edge" action={<StatusBadge status={c.status} />}>
      <div className="row wrap" style={{ gap: 8 }}>
        <span className="code-tag">{c.code}</span>
        <span className="badge">{c.departmentName}</span>
        <span className="badge">{faNumber(c.credits)} واحد</span>
      </div>
      <h2 className="current-title">
        <Link to={`/courses/${c.slug}`}>{c.title}</Link>
      </h2>
      <p className="text-2 small">{c.summary}</p>
      <div style={{ margin: '14px 0 6px' }} className="row between small">
        <span>
          {faNumber(c.lessonsDone)} از {faNumber(c.lessonCount)} Lesson
        </span>
        <span className="muted">{faPercent(c.percent)}</span>
      </div>
      <ProgressBar value={c.percent} label="پیشرفت درس" />
      <ol className="lesson-steps">
        {c.lessons.map((l) => (
          <li key={l.id} className={l.completed ? 'done' : l.id === c.nextLessonId ? 'next' : ''}>
            <Link to={`/lessons/${l.id}`}>
              <span className="step-dot">{l.completed ? <Icon name="check" /> : faNumber(l.order)}</span>
              <span>{l.title}</span>
            </Link>
          </li>
        ))}
      </ol>
      {next && (
        <Link to={`/lessons/${next.id}`} className="btn mint" style={{ marginTop: 6 }}>
          <Icon name="play" /> ادامه: {next.title}
        </Link>
      )}
    </Card>
  )
}

function MissionCard({ d }: { d: DashboardData }) {
  const m = d.mission
  const log = useLogStudy()
  const toast = useToast()
  const remaining = Math.max(0, m.targetMinutes - m.progressMinutes)
  const record = (minutes: number) =>
    log.mutate(
      { minutes, courseSlug: m.courseSlug },
      {
        onSuccess: (r) =>
          toast(r.missionCompletedNow ? `مأموریت امروز کامل شد! ${faNumber(m.xpReward)}+ XP` : `${faDuration(minutes)} مطالعه ثبت شد.`),
        onError: (e) => toast(e.message),
      },
    )
  return (
    <Card title="مأموریت امروز" icon="target" className={m.completed ? 'mint-edge' : 'brass-edge'} action={<span className="badge brass">{faNumber(m.xpReward)}+ XP</span>}>
      <p className="mission-text">{m.description}</p>
      <div className="row between small" style={{ margin: '14px 0 6px' }}>
        <span>
          {faNumber(m.progressMinutes)} از {faNumber(m.targetMinutes)} دقیقه
        </span>
        <span className="muted">{faPercent(m.percent)}</span>
      </div>
      <ProgressBar value={m.percent} label="پیشرفت مأموریت" />
      {m.completed ? (
        <p className="small" style={{ color: 'var(--mint)', marginTop: 14 }}>
          <Icon name="check" size={16} /> انجام شد؛ فردا مأموریت تازه‌ای در انتظارت است.
        </p>
      ) : (
        <>
          <p className="tiny muted" style={{ marginTop: 12 }}>
            {faNumber(remaining)} دقیقه‌ی دیگر مانده. زمان مطالعه‌ات را ثبت کن:
          </p>
          <div className="chip-group" style={{ marginTop: 8 }}>
            {[10, 15, 30].map((min) => (
              <button key={min} className="chip" disabled={log.isPending} onClick={() => record(min)}>
                {faNumber(min)} دقیقه
              </button>
            ))}
          </div>
        </>
      )}
    </Card>
  )
}

const CHECK_IN_OPTIONS = [10, 30, 60]

function CheckIn({ d }: { d: DashboardData }) {
  const mutation = useCheckIn()
  const current = mutation.data ?? d.checkIn
  return (
    <Card title="امروز چقدر وقت داری؟" icon="clock">
      <div className="chip-group" role="group" aria-label="زمان در دسترس">
        {CHECK_IN_OPTIONS.map((min) => (
          <button key={min} className="chip" aria-pressed={current?.minutes === min} disabled={mutation.isPending} onClick={() => mutation.mutate(min)}>
            {faNumber(min)} دقیقه
          </button>
        ))}
      </div>
      {mutation.isError && (
        <p className="small" style={{ color: 'var(--danger)', marginTop: 12 }}>
          {mutation.error.message}
        </p>
      )}
      {current ? (
        <div className="suggestion">
          <div className="tiny muted">پیشنهاد امروز</div>
          <strong>{current.suggestion.title}</strong>
          <p className="small text-2">{current.suggestion.description}</p>
          {current.suggestion.lessonId && (
            <Link className="btn sm mint" to={`/lessons/${current.suggestion.lessonId}`}>
              شروع <Icon name="arrowLeft" />
            </Link>
          )}
          {current.suggestion.termSlug && (
            <Link className="btn sm" to={`/dictionary?term=${current.suggestion.termSlug}`}>
              رفتن به دانشنامه <Icon name="arrowLeft" />
            </Link>
          )}
        </div>
      ) : (
        <p className="small muted" style={{ marginTop: 14 }}>
          یکی از گزینه‌ها را انتخاب کن تا بر اساس وضعیت درسی‌ات پیشنهاد بگیری.
        </p>
      )}
    </Card>
  )
}

function AdvisorCard({ d }: { d: DashboardData }) {
  if (!d.advisor) return null
  const a = d.advisor
  return (
    <Card title="استاد راهنما" icon="user" className="advisor">
      <div className="row">
        <span className="avatar advisor-avatar">{a.name.replace('دکتر ', '').replace('مهندس ', '').charAt(0)}</span>
        <div>
          <strong className="small">{a.name}</strong>
          <div className="tiny muted">{a.title}</div>
        </div>
      </div>
      <blockquote className="advice">{a.advice.message}</blockquote>
      {a.advice.focusDepartment && (
        <Link className="card-link" to={`/departments/${a.advice.focusDepartment}`}>
          مشاهده‌ی درس‌های پیشنهادی ←
        </Link>
      )}
    </Card>
  )
}

function ProjectCard({ d }: { d: DashboardData }) {
  const p = d.project
  if (!p) return null
  return (
    <Card title="پروژه‌ی پایانی سال" icon="code" action={<StatusBadge status={p.status} grade={p.grade} />}>
      <span className="code-tag">{p.code}</span>
      <h3 style={{ margin: '10px 0 6px', fontSize: '1rem' }}>
        <Link to={`/courses/${p.slug}`}>{p.title}</Link>
      </h3>
      <div className="row between small" style={{ margin: '10px 0 6px' }}>
        <span>
          {faNumber(p.lessonsDone)} از {faNumber(p.lessonCount)} مرحله
        </span>
        {p.grade != null ? <Grade value={p.grade} /> : <span className="muted">{faPercent(p.percent)}</span>}
      </div>
      <ProgressBar value={p.percent} tone="brass" label="پیشرفت پروژه" />
      {p.status === 'locked' && <p className="tiny muted" style={{ marginTop: 10 }}>با گذراندن پیش‌نیازها باز می‌شود.</p>}
    </Card>
  )
}

function PromotionCard({ d }: { d: DashboardData }) {
  const p = d.promotion
  return (
    <Card
      title={`شرایط ارتقا از ${d.student.levelName}`}
      icon="shield"
      action={
        <span className={`badge ${p.eligible ? 'done' : ''}`}>
          {faNumber(p.rules.filter((r) => r.met).length)}/{faNumber(p.rules.length)}
        </span>
      }
    >
      <ul className="list">
        {p.rules.map((r) => (
          <li key={r.key} style={{ alignItems: 'flex-start' }}>
            <span className={`rule-icon ${r.met ? 'met' : ''}`}>
              <Icon name={r.met ? 'check' : 'clock'} />
            </span>
            <div style={{ minWidth: 0 }}>
              <div className="small" style={{ fontWeight: 600 }}>
                {r.title}
              </div>
              {!r.met && r.blocking.length > 0 && <div className="tiny muted">مانده: {r.blocking.join('، ')}</div>}
            </div>
          </li>
        ))}
      </ul>
      <Link className="card-link" to="/levels">
        قوانین ارتقا ←
      </Link>
    </Card>
  )
}
