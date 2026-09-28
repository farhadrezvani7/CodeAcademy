import { Link } from 'react-router-dom'
import { useDepartments } from '../api/hooks'
import { QueryState } from '../components/QueryState'
import { PageHeader, ProgressBar } from '../components/ui'
import { faNumber, faPercent } from '../lib/format'

export default function Departments() {
  const query = useDepartments()
  return (
    <>
      <PageHeader eyebrow="ساختار دانشگاه" title="دانشکده‌ها" lead="چهار دانشکده‌ی کد آکادمی؛ هر کدام با زنجیره‌ای از درس‌ها از سال اول تا ششم." />
      <QueryState query={query} isEmpty={(d) => d.length === 0}>
        {(depts) => (
          <div className="grid grid-2">
            {depts.map((d) => (
              <Link key={d.slug} to={`/departments/${d.slug}`} className={`card dept-card big accent-${d.accent}`}>
                <div className="dept-bar" />
                <div className="row between">
                  <h2 className="serif">{d.nameFa}</h2>
                  <span className="code-tag">{d.codePrefix}</span>
                </div>
                <p className="text-2 small" style={{ margin: '8px 0 14px' }}>
                  {d.description}
                </p>
                <div className="row wrap small muted" style={{ gap: 14, marginBottom: 10 }}>
                  <span>{faNumber(d.courseCount)} درس</span>
                  {d.projectCount > 0 && <span>{faNumber(d.projectCount)} پروژه</span>}
                  <span>{faNumber(d.credits)} واحد</span>
                  <span>
                    {faNumber(d.creditsPassed)} واحد گذرانده ({faPercent(d.percent)})
                  </span>
                </div>
                <ProgressBar value={d.percent} label={d.nameFa} />
              </Link>
            ))}
          </div>
        )}
      </QueryState>
    </>
  )
}
