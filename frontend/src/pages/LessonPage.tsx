import { useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { useCompleteLesson, useLesson } from '../api/hooks'
import type { CompleteLessonResult, Lesson } from '../api/types'
import { CodeBlock } from '../components/CodeBlock'
import { Icon } from '../components/Icon'
import { QueryState, Skeleton } from '../components/QueryState'
import { Inline, RichText } from '../components/RichText'
import { useToast } from '../components/Toast'
import { StatusBadge } from '../components/ui'
import { faGrade, faNumber } from '../lib/format'

const SECTIONS = [
  ['what', 'این چیست؟'],
  ['why', 'چرا مهم است؟'],
  ['simple', 'مثال ساده'],
  ['real', 'مثال در پروژه واقعی'],
  ['mistakes', 'اشتباهات رایج'],
  ['practices', 'بهترین شیوه‌ها'],
  ['terms', 'مفاهیم مرتبط'],
  ['try', 'تمرین کوتاه'],
] as const

export default function LessonPage() {
  const { id = '' } = useParams()
  const query = useLesson(id)
  return (
    <QueryState query={query} loading={<Skeleton height={220} count={3} />}>
      {(lesson) => <LessonView key={lesson.id} lesson={lesson} />}
    </QueryState>
  )
}

function Section({ n, id, title, children }: { n: number; id: string; title: string; children: React.ReactNode }) {
  return (
    <section className="lesson-section" id={id}>
      <h2>
        <span className="section-no">{faNumber(n)}</span>
        {title}
      </h2>
      {children}
    </section>
  )
}

function LessonView({ lesson }: { lesson: Lesson }) {
  return (
    <div className="lesson-layout">
      <article className="lesson">
        <nav className="crumbs tiny">
          <Link to={`/departments/${lesson.course.departmentSlug}`}>{lesson.course.departmentName}</Link>
          <span>/</span>
          <Link to={`/courses/${lesson.course.slug}`}>
            {lesson.course.code} — {lesson.course.title}
          </Link>
        </nav>
        <header className="lesson-head">
          <div className="eyebrow">
            Lesson {faNumber(lesson.order)} از {faNumber(lesson.lessonCount)} · {faNumber(lesson.estimatedMinutes)} دقیقه
          </div>
          <h1>{lesson.title}</h1>
          <div className="row wrap" style={{ marginTop: 8 }}>
            <StatusBadge status={lesson.course.status} />
            {lesson.completed && (
              <span className="badge done">
                <Icon name="check" /> تکمیل‌شده · خودارزیابی {faGrade(lesson.selfScore)}
              </span>
            )}
          </div>
        </header>

        {lesson.locked && (
          <div className="state-box" style={{ marginBottom: 20 }}>
            <Icon name="lock" />
            <h3>این درس هنوز برایت باز نشده است</h3>
            <p className="small">می‌توانی محتوا را مرور کنی، اما ثبت تکمیل بعد از گذراندن پیش‌نیازها ممکن است.</p>
          </div>
        )}

        <Section n={1} id="what" title="این چیست؟">
          <RichText text={lesson.what} />
        </Section>
        <Section n={2} id="why" title="چرا مهم است؟">
          <RichText text={lesson.why} />
        </Section>
        <Section n={3} id="simple" title="مثال ساده">
          <CodeBlock code={lesson.simpleExample.code} language={lesson.simpleExample.language} />
          <p className="example-note">
            <Inline text={lesson.simpleExample.explanation} />
          </p>
        </Section>
        <Section n={4} id="real" title="مثال در پروژه واقعی">
          <CodeBlock code={lesson.realExample.code} language={lesson.realExample.language} />
          <p className="example-note">
            <Inline text={lesson.realExample.explanation} />
          </p>
        </Section>
        <Section n={5} id="mistakes" title="اشتباهات رایج">
          <ul className="bullets bad">
            {lesson.mistakes.map((m, i) => (
              <li key={i}>
                <Icon name="x" />
                <span>
                  <Inline text={m} />
                </span>
              </li>
            ))}
          </ul>
        </Section>
        <Section n={6} id="practices" title="بهترین شیوه‌ها">
          <ul className="bullets good">
            {lesson.bestPractices.map((m, i) => (
              <li key={i}>
                <Icon name="check" />
                <span>
                  <Inline text={m} />
                </span>
              </li>
            ))}
          </ul>
        </Section>
        <Section n={7} id="terms" title="مفاهیم مرتبط">
          <div className="row wrap" style={{ gap: 8 }}>
            {lesson.relatedTerms.map((t) => (
              <Link key={t.slug} to={`/dictionary?term=${t.slug}`} className="term-link">
                <span className="ltr">{t.nameEn}</span>
                <span className="tiny muted">{t.nameFa}</span>
              </Link>
            ))}
          </div>
        </Section>
        <Section n={8} id="try" title="تمرین کوتاه">
          <div className="try-it">
            <Icon name="pencil" />
            <RichText text={lesson.tryIt} />
          </div>
        </Section>

        <CompletePanel lesson={lesson} />

        <nav className="lesson-nav">
          {lesson.previous ? (
            <Link to={`/lessons/${lesson.previous.id}`} className="btn ghost">
              <Icon name="arrowRight" /> {lesson.previous.title}
            </Link>
          ) : (
            <span />
          )}
          {lesson.next && (
            <Link to={`/lessons/${lesson.next.id}`} className="btn ghost">
              {lesson.next.title} <Icon name="arrowLeft" />
            </Link>
          )}
        </nav>
      </article>

      <aside className="lesson-toc">
        <div className="card">
          <div className="tiny muted" style={{ marginBottom: 8 }}>
            ساختار Lesson
          </div>
          <ol>
            {SECTIONS.map(([id, title], i) => (
              <li key={id}>
                <a href={`#${id}`}>
                  <span>{faNumber(i + 1)}</span> {title}
                </a>
              </li>
            ))}
          </ol>
        </div>
      </aside>
    </div>
  )
}

function CompletePanel({ lesson }: { lesson: Lesson }) {
  const [score, setScore] = useState<number | null>(lesson.selfScore)
  const [result, setResult] = useState<CompleteLessonResult | null>(null)
  const mutation = useCompleteLesson(lesson.id)
  const toast = useToast()

  if (lesson.locked) return null

  const submit = () => {
    if (score == null) return
    mutation.mutate(score, {
      onSuccess: (r) => {
        setResult(r)
        const parts = [r.xpGained > 0 ? `${faNumber(r.xpGained)}+ XP` : 'خودارزیابی به‌روز شد']
        if (r.promotedTo) parts.push(`ارتقا به ${r.promotedTo}!`)
        if (r.newAchievements.length) parts.push(`دستاورد: ${r.newAchievements.join('، ')}`)
        toast(parts.join(' · '))
      },
    })
  }

  return (
    <section className="complete-panel card brass-edge">
      <h2 className="serif" style={{ fontSize: '1.1rem' }}>
        {lesson.completed ? 'به‌روزرسانی خودارزیابی' : 'ثبت تکمیل Lesson'}
      </h2>
      <p className="small text-2" style={{ margin: '4px 0 12px' }}>
        بعد از انجام تمرین کوتاه، صادقانه ارزیابی کن. نمره‌ی درس میانگین خودارزیابی Lessonهای آن است و حداقل نمره‌ی قبولی ۱۴ است.
      </p>
      <div className="score-options" role="radiogroup" aria-label="خودارزیابی">
        {lesson.scoreOptions.map((o) => (
          <button key={o.score} role="radio" aria-checked={score === o.score} className="score-option" onClick={() => setScore(o.score)}>
            <strong>{faGrade(o.score)}</strong>
            <span className="small">{o.label}</span>
          </button>
        ))}
      </div>
      {mutation.isError && (
        <p className="small" role="alert" style={{ color: 'var(--danger)', marginTop: 10 }}>
          {mutation.error.message}
        </p>
      )}
      <button className="btn primary" style={{ marginTop: 14 }} disabled={score == null || mutation.isPending} onClick={submit}>
        {mutation.isPending ? 'در حال ثبت…' : lesson.completed ? 'ذخیره‌ی خودارزیابی' : `تکمیل Lesson و دریافت ${faNumber(25)} XP`}
      </button>
      {result && (
        <div className="result-box">
          {result.xpGained > 0 && <span className="badge brass">{faNumber(result.xpGained)}+ XP</span>}
          {result.courseStatus === 'done' && result.courseGrade != null && (
            <span className={`badge ${result.courseGrade >= 14 ? 'done' : 'fail'}`}>نمره‌ی درس: {faGrade(result.courseGrade)}</span>
          )}
          {result.missionCompleted && <span className="badge done">مأموریت امروز کامل شد</span>}
          {result.unlockedCourses.length > 0 && <span className="badge enrolled">درس‌های باز شده: {result.unlockedCourses.join('، ')}</span>}
          {result.promotedTo && <span className="badge brass">ارتقا به {result.promotedTo}</span>}
          {result.graduated && <span className="badge brass">فارغ‌التحصیل شدی!</span>}
          {result.newAchievements.map((a) => (
            <span key={a} className="badge brass">
              <Icon name="trophy" /> {a}
            </span>
          ))}
        </div>
      )}
    </section>
  )
}
