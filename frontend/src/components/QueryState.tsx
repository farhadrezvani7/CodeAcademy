import type { UseQueryResult } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import { ApiError } from '../api/client'
import { Icon } from './Icon'

export function Skeleton({ height = 120, count = 1 }: { height?: number; count?: number }) {
  return (
    <div className="stack" aria-busy="true" aria-label="در حال بارگذاری">
      {Array.from({ length: count }, (_, i) => (
        <div key={i} className="skeleton" style={{ height }} />
      ))}
    </div>
  )
}

export function ErrorState({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  const message = error instanceof ApiError ? error.message : 'مشکلی پیش آمد. دوباره تلاش کن.'
  const notFound = error instanceof ApiError && error.status === 404
  return (
    <div className="state-box error" role="alert">
      <Icon name="alert" />
      <h3>{notFound ? 'پیدا نشد' : 'بارگذاری ناموفق بود'}</h3>
      <p className="small">{message}</p>
      {onRetry && !notFound && (
        <button className="btn sm" onClick={onRetry}>
          تلاش دوباره
        </button>
      )}
    </div>
  )
}

export function EmptyState({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <div className="state-box">
      <Icon name="inbox" />
      <h3>{title}</h3>
      {children && <p className="small">{children}</p>}
    </div>
  )
}

/**
 * Renders loading / error / empty / success for a query. `isEmpty` decides
 * when successful data should show the empty state.
 */
export function QueryState<T>({
  query,
  children,
  loading,
  isEmpty,
  empty,
}: {
  query: UseQueryResult<T>
  children: (data: T) => ReactNode
  loading?: ReactNode
  isEmpty?: (data: T) => boolean
  empty?: ReactNode
}) {
  if (query.isPending) return <>{loading ?? <Skeleton height={140} count={3} />}</>
  if (query.isError) return <ErrorState error={query.error} onRetry={() => query.refetch()} />
  if (isEmpty?.(query.data)) return <>{empty ?? <EmptyState title="موردی برای نمایش وجود ندارد" />}</>
  return <>{children(query.data)}</>
}
