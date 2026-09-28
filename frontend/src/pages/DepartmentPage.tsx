import { Link, useParams } from 'react-router-dom'
import { useDepartment } from '../api/hooks'
import { Accordion } from '../components/Accordion'
import { QueryState } from '../components/QueryState'
import { PageHeader, ProgressBar } from '../components/ui'
import { faNumber, faPercent } from '../lib/format'
import { CourseBody, CourseHeader } from './CourseParts'

export default function DepartmentPage() {
  const { slug = '' } = useParams()
  const query = useDepartment(slug)
  return (
    <QueryState query={query}>
      {(d) => {
        const years = [...new Set(d.courses.map((c) => c.year))]
        return (
          <>
            <PageHeader
              eyebrow={<Link to="/departments">دانشکده‌ها</Link>}
              title={`دانشکده ${d.nameFa}`}
              lead={d.description}
            />
            <div className={`card accent-${d.accent}`} style={{ marginBottom: 22 }}>
              <div className="row wrap between small">
                <span>
                  {faNumber(d.courseCount)} درس · {faNumber(d.projectCount)} پروژه · {faNumber(d.credits)} واحد
                </span>
                <span className="muted">
                  {faNumber(d.creditsPassed)} واحد گذرانده · {faPercent(d.percent)}
                </span>
              </div>
              <div style={{ marginTop: 8 }}>
                <ProgressBar value={d.percent} label="پیشرفت دانشکده" />
              </div>
            </div>
            {years.map((year) => (
              <section key={year}>
                <h2 className="section-title">
                  سال {faNumber(year)} · {d.courses.find((c) => c.year === year)?.levelName}
                </h2>
                {d.courses
                  .filter((c) => c.year === year)
                  .map((c) => (
                    <Accordion key={c.slug} header={<CourseHeader c={c} />} defaultOpen={c.status === 'in_progress'}>
                      <CourseBody c={c} />
                    </Accordion>
                  ))}
              </section>
            ))}
          </>
        )
      }}
    </QueryState>
  )
}
