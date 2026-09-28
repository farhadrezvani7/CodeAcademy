import type { ReactNode } from 'react'

const KEYWORDS = new Set(
  (
    'abstract as assert async await base break case catch class const continue covariant default deferred do dynamic else enum export ' +
    'extends extension external factory false final finally for Function get hide if implements import in interface is late library mixin new null ' +
    'on operator part required rethrow return sealed set show static super switch sync this throw true try typedef var void when while with yield ' +
    'git cd npm flutter dart run on jobs steps uses name'
  ).split(' '),
)

const TOKEN = /(\/\/[^\n]*|#[^\n]*|\/\*[\s\S]*?\*\/)|('(?:\\.|[^'\\\n])*'|"(?:\\.|[^"\\\n])*")|\b(\d+(?:\.\d+)?)\b|\b([A-Z][A-Za-z0-9_]*)\b|\b([a-z_][A-Za-z0-9_]*)\b/g

/** Minimal tokenizer producing React nodes (never HTML strings). */
export function highlight(code: string, language: string): ReactNode[] {
  if (language === 'text' || language === 'markdown') return [code]
  const out: ReactNode[] = []
  let last = 0
  let i = 0
  for (const m of code.matchAll(TOKEN)) {
    const start = m.index ?? 0
    if (start > last) out.push(code.slice(last, start))
    const [text, comment, str, num, type, word] = m
    // `#` starts a comment only in shell/yaml.
    if (comment && comment.startsWith('#') && !['bash', 'yaml'].includes(language)) {
      out.push(text)
    } else if (comment) out.push(<span key={i++} className="tok-c">{text}</span>)
    else if (str) out.push(<span key={i++} className="tok-s">{text}</span>)
    else if (num) out.push(<span key={i++} className="tok-n">{text}</span>)
    else if (type) out.push(<span key={i++} className="tok-t">{text}</span>)
    else if (word && KEYWORDS.has(word)) out.push(<span key={i++} className="tok-k">{text}</span>)
    else out.push(text)
    last = start + text.length
  }
  if (last < code.length) out.push(code.slice(last))
  return out
}

