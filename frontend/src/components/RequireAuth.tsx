import type { ReactNode } from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useSession } from '../api/hooks'
import { ErrorState, Skeleton } from './QueryState'

/** Renders children for a signed-in student; otherwise redirects to login. */
export function RequireAuth({ children }: { children: ReactNode }) {
  const session = useSession()
  const location = useLocation()
  if (session.isPending) return <Skeleton height={160} count={3} />
  if (session.isError) return <ErrorState error={session.error} onRetry={() => session.refetch()} />
  if (!session.data) {
    const next = encodeURIComponent(location.pathname + location.search)
    return <Navigate to={`/login?next=${next}`} replace />
  }
  return <>{children}</>
}
