import { useId, useState, type ReactNode } from 'react'
import { Icon } from './Icon'

export function Accordion({ header, children, defaultOpen = false }: { header: ReactNode; children: ReactNode; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen)
  const id = useId()
  return (
    <div className={`accordion ${open ? 'open' : ''}`}>
      <button className="accordion-trigger" aria-expanded={open} aria-controls={id} onClick={() => setOpen((o) => !o)}>
        {header}
        <Icon name="chevron" className="chev" />
      </button>
      {open && (
        <div className="accordion-panel" id={id} role="region">
          {children}
        </div>
      )}
    </div>
  )
}
