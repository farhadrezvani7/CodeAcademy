import { Fragment, useState } from 'react'
import { highlight } from '../lib/highlight'

export function CodeBlock({ code, language }: { code: string; language: string }) {
  return (
    <div className="code">
      <div className="code-head">
        <span>{language}</span>
        <CopyButton text={code} />
      </div>
      <pre>
        <code>
          {highlight(code, language).map((n, i) => (
            <Fragment key={i}>{n}</Fragment>
          ))}
        </code>
      </pre>
    </div>
  )
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)
  const copy = () =>
    navigator.clipboard
      ?.writeText(text)
      .then(() => {
        setCopied(true)
        setTimeout(() => setCopied(false), 1500)
      })
      .catch(() => undefined)
  return (
    <button className="code-copy" onClick={copy} aria-label="کپی کد">
      {copied ? 'کپی شد ✓' : 'کپی'}
    </button>
  )
}
