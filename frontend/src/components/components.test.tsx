import type { UseQueryResult } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ApiError } from '../api/client'
import { Accordion } from './Accordion'
import { highlight } from '../lib/highlight'
import { QueryState } from './QueryState'
import { Inline } from './RichText'
import { ProgressBar, StatusBadge } from './ui'

const q = <T,>(state: Partial<UseQueryResult<T>>) => state as UseQueryResult<T>

describe('Accordion', () => {
  it('toggles its panel', async () => {
    render(
      <Accordion header="نرم-۱۰۱">
        <p>محتوای درس</p>
      </Accordion>,
    )
    const trigger = screen.getByRole('button', { name: /نرم-۱۰۱/ })
    expect(trigger).toHaveAttribute('aria-expanded', 'false')
    expect(screen.queryByText('محتوای درس')).not.toBeInTheDocument()
    await userEvent.click(trigger)
    expect(trigger).toHaveAttribute('aria-expanded', 'true')
    expect(screen.getByText('محتوای درس')).toBeInTheDocument()
    await userEvent.click(trigger)
    expect(screen.queryByText('محتوای درس')).not.toBeInTheDocument()
  })
})

describe('QueryState', () => {
  it('shows loading', () => {
    render(<QueryState query={q({ isPending: true })}>{() => 'data'}</QueryState>)
    expect(screen.getByLabelText('در حال بارگذاری')).toBeInTheDocument()
  })

  it('shows the API error message with retry', async () => {
    const refetch = vi.fn()
    render(
      <QueryState query={q({ isPending: false, isError: true, error: new ApiError(500, 'internal_error', 'خطای داخلی سرور'), refetch })}>
        {() => 'data'}
      </QueryState>,
    )
    expect(screen.getByRole('alert')).toHaveTextContent('خطای داخلی سرور')
    await userEvent.click(screen.getByRole('button', { name: 'تلاش دوباره' }))
    expect(refetch).toHaveBeenCalled()
  })

  it('shows empty and success states', () => {
    const { rerender } = render(
      <QueryState query={q({ isPending: false, isError: false, data: [] as number[] })} isEmpty={(d) => d.length === 0}>
        {() => 'data'}
      </QueryState>,
    )
    expect(screen.getByText('موردی برای نمایش وجود ندارد')).toBeInTheDocument()
    rerender(
      <QueryState query={q({ isPending: false, isError: false, data: [1] })} isEmpty={(d) => d.length === 0}>
        {(d) => `تعداد ${d.length}`}
      </QueryState>,
    )
    expect(screen.getByText('تعداد 1')).toBeInTheDocument()
  })
})

describe('ui primitives', () => {
  it('progress bar exposes Persian value text', () => {
    render(<ProgressBar value={75} label="پیشرفت" />)
    expect(screen.getByRole('progressbar', { name: 'پیشرفت' })).toHaveAttribute('aria-valuetext', '۷۵٪')
  })

  it('status badge marks failing grades', () => {
    render(<StatusBadge status="done" grade={12} />)
    expect(screen.getByText(/مردود/)).toBeInTheDocument()
  })

  it('renders inline code without HTML injection', () => {
    const { container } = render(<Inline text="از `Future<int>` و <b>x</b> استفاده کن" />)
    expect(container.querySelector('code')).toHaveTextContent('Future<int>')
    expect(container.querySelector('b')).toBeNull()
  })

  it('highlights keywords and strings', () => {
    const { container } = render(<pre>{highlight("final name = 'Ali'; // hi", 'dart')}</pre>)
    expect(container.querySelector('.tok-k')).toHaveTextContent('final')
    expect(container.querySelector('.tok-s')).toHaveTextContent("'Ali'")
    expect(container.querySelector('.tok-c')).toHaveTextContent('// hi')
  })
})
