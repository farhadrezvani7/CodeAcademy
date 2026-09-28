import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from 'react'

type ToastFn = (message: ReactNode) => void
const ToastContext = createContext<ToastFn>(() => undefined)

export function ToastProvider({ children }: { children: ReactNode }) {
  const [message, setMessage] = useState<ReactNode>(null)
  const show = useCallback<ToastFn>((m) => setMessage(m), [])

  useEffect(() => {
    if (!message) return
    const t = setTimeout(() => setMessage(null), 6000)
    return () => clearTimeout(t)
  }, [message])

  return (
    <ToastContext.Provider value={show}>
      {children}
      {message && (
        <div className="toast" role="status" onClick={() => setMessage(null)}>
          {message}
        </div>
      )}
    </ToastContext.Provider>
  )
}

export const useToast = () => useContext(ToastContext)
