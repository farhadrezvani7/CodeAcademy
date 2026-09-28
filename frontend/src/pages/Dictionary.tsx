import { useDeferredValue, useEffect, useRef, useState } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import { useDictionary } from '../api/hooks'
import { CodeBlock } from '../components/CodeBlock'
import { Icon } from '../components/Icon'
import { EmptyState, QueryState } from '../components/QueryState'
import { Inline } from '../components/RichText'
import { PageHeader } from '../components/ui'
import { faNumber } from '../lib/format'

export default function Dictionary() {
  const [params, setParams] = useSearchParams()
  const focus = params.get('term')
  const [input, setInput] = useState(params.get('q') ?? '')
  const q = useDeferredValue(input.trim())
  const query = useDictionary(q)
  const focused = useRef<HTMLElement | null>(null)

  useEffect(() => {
    focused.current?.scrollIntoView({ block: 'center', behavior: 'smooth' })
  }, [focus, query.data])

  return (
    <>
      <PageHeader eyebrow="مرجع مفاهیم" title="دانشنامه" lead="اصطلاحات فنی با معادل فارسی، تعریف کوتاه و مفاهیم مرتبط. جست‌وجو هم به انگلیسی کار می‌کند و هم به فارسی." />
      <div className="search" style={{ marginBottom: 18 }}>
        <Icon name="search" />
        <input
          className="input"
          type="search"
          value={input}
          maxLength={64}
          onChange={(e) => {
            setInput(e.target.value)
            if (focus) setParams({}, { replace: true })
          }}
          placeholder="مثلاً Stream، تزریق وابستگی یا تست…"
          aria-label="جست‌وجو در دانشنامه"
        />
      </div>
      <QueryState
        query={query}
        isEmpty={(terms) => terms.length === 0}
        empty={<EmptyState title="اصطلاحی پیدا نشد">عبارت دیگری را امتحان کن.</EmptyState>}
      >
        {(terms) => (
          <>
            <p className="tiny muted" style={{ marginBottom: 10 }} aria-live="polite">
              {faNumber(terms.length)} اصطلاح {query.isFetching && '· در حال جست‌وجو…'}
            </p>
            <div className="grid grid-2">
              {terms.map((t) => (
                <article
                  key={t.slug}
                  id={`term-${t.slug}`}
                  ref={t.slug === focus ? (el) => void (focused.current = el) : undefined}
                  className={`card term ${t.slug === focus ? 'focused' : ''}`}
                >
                  <div className="row between wrap">
                    <h2 className="ltr term-en">{t.nameEn}</h2>
                    <span className="badge">{t.categoryName}</span>
                  </div>
                  <div className="term-fa serif">{t.nameFa}</div>
                  <p className="small text-2" style={{ marginTop: 8 }}>
                    <Inline text={t.definition} />
                  </p>
                  {t.example && (
                    <div style={{ marginTop: 10 }}>
                      <CodeBlock code={t.example} language="dart" />
                    </div>
                  )}
                  {t.related.length > 0 && (
                    <div className="row wrap" style={{ gap: 6, marginTop: 12 }}>
                      <span className="tiny muted">مرتبط:</span>
                      {t.related.map((r) => (
                        <Link key={r.slug} to={`/dictionary?term=${r.slug}`} onClick={() => setInput('')} className="badge ltr">
                          {r.nameEn}
                        </Link>
                      ))}
                    </div>
                  )}
                </article>
              ))}
            </div>
          </>
        )}
      </QueryState>
    </>
  )
}
