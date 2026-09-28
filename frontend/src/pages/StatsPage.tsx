import { useEffect, useRef } from 'react'
import { useStats } from '../api/hooks'
import type { DayMinutes, Stats } from '../api/types'
import { QueryState } from '../components/QueryState'
import { Card, PageHeader, Stat } from '../components/ui'
import { faDuration, faNumber, faPercent, jalali, jalaliDayMonth, jalaliMonthName, parseDate, weekdayShort } from '../lib/format'

export default function StatsPage() {
  const query = useStats()
  return (
    <>
      <PageHeader eyebrow="گزارش مطالعه" title="آمار یادگیری" lead="ریتم مطالعه‌ات در شش ماه اخیر، هفت روز گذشته و سهم هر دانشکده از زمانی که گذاشته‌ای." />
      <QueryState query={query}>{(s) => <StatsView s={s} />}</QueryState>
    </>
  )
}

function StatsView({ s }: { s: Stats }) {
  return (
    <div className="stack" style={{ gap: 18 }}>
      {s.totals.minutes === 0 && (
        <div className="notice" role="status">
          <span>هنوز فعالیتی ثبت نکرده‌ای. با تکمیل اولین Lesson یا ثبت زمان مطالعه در داشبورد، این نمودارها پر می‌شوند.</span>
        </div>
      )}
      <div className="card stats-tiles five">
        <Stat label="کل زمان مطالعه" icon="clock" value={faNumber(Math.round(s.totals.minutes / 60))} unit="ساعت" tone="brass" />
        <Stat label="روزهای فعال" icon="target" value={faNumber(s.totals.activeDays)} unit="روز" />
        <Stat label="میانگین هر روز فعال" icon="chart" value={faNumber(s.totals.averagePerActiveDay)} unit="دقیقه" />
        <Stat label="Streak فعلی / بیشترین" icon="flame" value={`${faNumber(s.totals.currentStreak)} / ${faNumber(s.totals.longestStreak)}`} unit="روز" tone="mint" />
        <Stat label="Lessonهای کامل‌شده" icon="book" value={faNumber(s.totals.lessonsCompleted)} />
      </div>
      <Card title="تقویم فعالیت" icon="dashboard">
        <Heatmap days={s.heatmap} />
      </Card>
      <div className="grid grid-2">
        <Card title="فعالیت هفتگی" icon="chart">
          <WeeklyBars days={s.weekly} />
        </Card>
        <Card title="زمان بر اساس موضوع" icon="building">
          <div className="stack">
            {s.byTopic.map((t) => (
              <div key={t.slug} className={`topic accent-${t.accent}`}>
                <div className="row between small">
                  <span>{t.nameFa}</span>
                  <span className="muted tiny">
                    {faDuration(t.minutes)} · {faPercent(t.percent)}
                  </span>
                </div>
                <div className="bar">
                  <span style={{ width: `${t.percent}%`, background: 'var(--accent)' }} />
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  )
}

function level(minutes: number): number {
  if (minutes <= 0) return 0
  if (minutes < 20) return 1
  if (minutes < 40) return 2
  if (minutes < 60) return 3
  return 4
}

export function Heatmap({ days }: { days: DayMinutes[] }) {
  const scrollRef = useRef<HTMLDivElement>(null)
  // In RTL the newest week is at the far (left) end; start scrolled there on narrow screens.
  useEffect(() => {
    const el = scrollRef.current
    if (el) el.scrollLeft = -el.scrollWidth
  }, [days])
  if (days.length === 0) return null
  // Persian weeks start on Saturday (JS day 6).
  const lead = (parseDate(days[0].date).getDay() + 1) % 7
  const cells: (DayMinutes | null)[] = [...Array<null>(lead).fill(null), ...days]
  const weeks: (DayMinutes | null)[][] = []
  for (let i = 0; i < cells.length; i += 7) weeks.push(cells.slice(i, i + 7))

  let lastMonth = ''
  return (
    <div className="heatmap-wrap" ref={scrollRef}>
      <div className="heatmap" role="img" aria-label="تقویم فعالیت روزانه">
        {weeks.map((week, wi) => {
          const first = week.find(Boolean)
          const month = first ? jalaliMonthName(first.date) : ''
          const showMonth = month !== lastMonth
          lastMonth = month
          return (
            <div key={wi} className="heat-col">
              <span className="heat-month">{showMonth ? month : ''}</span>
              {week.map((d, di) =>
                d ? (
                  <span key={di} className={`heat-cell l${level(d.minutes)}`} title={`${jalali(d.date)}: ${d.minutes ? faDuration(d.minutes) : 'بدون فعالیت'}`} />
                ) : (
                  <span key={di} className="heat-cell empty" />
                ),
              )}
            </div>
          )
        })}
      </div>
      <div className="row tiny muted" style={{ gap: 6, marginTop: 10, justifyContent: 'flex-end' }}>
        کمتر
        {[0, 1, 2, 3, 4].map((l) => (
          <span key={l} className={`heat-cell l${l}`} />
        ))}
        بیشتر
      </div>
    </div>
  )
}

export function WeeklyBars({ days }: { days: DayMinutes[] }) {
  const max = Math.max(30, ...days.map((d) => d.minutes))
  const total = days.reduce((a, d) => a + d.minutes, 0)
  return (
    <>
      <div className="weekly">
        {days.map((d) => (
          <div key={d.date} className="weekly-col" title={`${jalali(d.date)}: ${faDuration(d.minutes)}`}>
            <span className="tiny muted">{d.minutes ? faNumber(d.minutes) : ''}</span>
            <div className="weekly-track">
              <div className="weekly-bar" style={{ height: `${(d.minutes / max) * 100}%` }} />
            </div>
            <span className="tiny">{weekdayShort(d.date)}</span>
            <span className="tiny muted">{jalaliDayMonth(d.date)}</span>
          </div>
        ))}
      </div>
      <p className="small text-2" style={{ marginTop: 10 }}>
        مجموع هفت روز اخیر: {faDuration(total)}
      </p>
    </>
  )
}
