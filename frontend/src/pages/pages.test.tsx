import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Route, Routes } from 'react-router-dom'
import type { Dashboard as DashboardData, Term } from '../api/types'
import { Layout } from '../components/Layout'
import { mockApi, renderWithProviders } from '../test/utils'
import Dashboard from './Dashboard'
import Dictionary from './Dictionary'
import { RequireAuth } from '../components/RequireAuth'
import { LoginPage, RegisterPage } from './AuthPages'

afterEach(() => vi.unstubAllGlobals())

const term = (slug: string, nameEn: string, nameFa: string): Term => ({
  slug, nameEn, nameFa, category: 'dart', categoryName: 'Dart', definition: `تعریف ${nameEn}`, example: '', related: [],
})
const TERMS = [term('future', 'Future', 'نتیجه‌ی ناهمزمان'), term('stream', 'Stream', 'جریان داده'), term('solid', 'SOLID', 'اصول سالید')]

describe('Dictionary', () => {
  it('searches live through the API', async () => {
    const fetch = mockApi({
      '/dictionary': (url) => {
        const q = (url.searchParams.get('q') ?? '').toLowerCase()
        return TERMS.filter((t) => !q || t.nameEn.toLowerCase().includes(q) || t.nameFa.includes(q))
      },
    })
    renderWithProviders(<Dictionary />)
    expect(await screen.findByText('Future')).toBeInTheDocument()
    expect(screen.getByText('SOLID')).toBeInTheDocument()

    await userEvent.type(screen.getByRole('searchbox'), 'stre')
    await waitFor(() => expect(screen.queryByText('SOLID')).not.toBeInTheDocument())
    expect(screen.getByText('Stream')).toBeInTheDocument()
    expect(fetch.mock.calls.some(([u]) => String(u).includes('q=stre'))).toBe(true)
  })

  it('shows the empty state', async () => {
    mockApi({ '/dictionary': () => [] })
    renderWithProviders(<Dictionary />)
    expect(await screen.findByText('اصطلاحی پیدا نشد')).toBeInTheDocument()
  })
})

const DASHBOARD: DashboardData = {
  today: '2026-09-28',
  student: { name: 'آرین کاظمی', studentId: 'CA-1', year: 3, currentTerm: 5, levelName: 'میدل', levelNameEn: 'Mid', levelBlurb: 'با API واقعی کار می‌کنی', totalXp: 1975, streakDays: 23, longestStreak: 23, gpa: 17.81, graduated: false },
  progress: { overallPercent: 31, creditsPassed: 31, totalCredits: 99, yearPercent: 0, yearCreditsPassed: 0, yearCredits: 18 },
  currentCourse: null,
  mission: { id: 1, date: '2026-09-28', description: 'امروز ۳۰ دقیقه روی درس معماری تمیز کار کن.', courseSlug: 'se-410', courseTitle: 'معماری تمیز', targetMinutes: 30, progressMinutes: 10, percent: 33, xpReward: 50, completed: false },
  checkIn: null,
  project: null,
  promotion: { year: 3, eligible: false, rules: [{ key: 'min_grade', title: 'حداقل نمره‌ی ۱۴ در هر درس', met: false, blocking: ['نرم-۳۱۰'] }] },
  recentAchievements: [],
  recentActivity: [],
  skills: [],
  advisor: { name: 'راهنمای آموزشی دانشکده Flutter', title: 'پیشنهاد بر اساس نمره‌ها', advice: { message: 'بهتر است این هفته کمی بیشتر روی Flutter تمرکز کنی.', focusDepartment: 'flutter', tone: 'focus' } },
  announcements: [],
  rank: { rank: 3, of: 7, levelName: 'میدل' },
}

describe('Dashboard', () => {
  it('renders the student state from the API', async () => {
    mockApi({ '/dashboard': () => DASHBOARD })
    renderWithProviders(<Dashboard />)
    expect(await screen.findByText('سلام آرین، خوش برگشتی')).toBeInTheDocument()
    expect(screen.getByText('۱٬۹۷۵')).toBeInTheDocument()
    expect(screen.getByText('۱۷٫۸۱')).toBeInTheDocument()
    expect(screen.getByText(DASHBOARD.mission.description)).toBeInTheDocument()
    expect(screen.getByText(/روی Flutter تمرکز کنی/)).toBeInTheDocument()
    expect(screen.getByText('۶ مهر ۱۴۰۵')).toBeInTheDocument()
  })

  it('check-in posts the chosen time and shows the suggestion', async () => {
    const fetch = mockApi({
      '/dashboard': () => DASHBOARD,
      '/check-in': () => ({ minutes: 30, suggestion: { kind: 'study_lesson', title: 'مطالعه‌ی درس «Stream»', description: 'درس بعدی را بخوان', lessonId: 9, termSlug: null } }),
    })
    renderWithProviders(<Dashboard />)
    const group = await screen.findByRole('group', { name: 'زمان در دسترس' })
    await userEvent.click(group.querySelectorAll('button')[1])
    expect(await screen.findByText('مطالعه‌ی درس «Stream»')).toBeInTheDocument()
    const post = fetch.mock.calls.find(([u]) => String(u).endsWith('/check-in'))
    expect(JSON.parse(String(post?.[1]?.body))).toEqual({ minutes: 30 })
  })

  it('keeps the page usable when the API fails', async () => {
    mockApi({ '/dashboard': () => new Response(JSON.stringify({ error: { code: 'internal_error', message: 'خطای داخلی سرور رخ داد.' } }), { status: 500 }) })
    renderWithProviders(<Dashboard />)
    expect(await screen.findByRole('alert')).toHaveTextContent('خطای داخلی سرور رخ داد.')
    expect(screen.getByRole('button', { name: 'تلاش دوباره' })).toBeInTheDocument()
  })
})

describe('Layout', () => {
  it('lists every section and toggles the mobile drawer', async () => {
    mockApi({ '/me': () => ({ id: 1, name: 'آرین', email: 'a@b.ir', studentId: 'x', year: 3, levelName: 'میدل', currentTerm: 5, totalXp: 10, streakDays: 1, joinedAt: '2025-01-01T00:00:00Z' }) })
    const { container } = renderWithProviders(
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<p>صفحه</p>} />
        </Route>
      </Routes>,
    )
    for (const label of ['خانه', 'داشبورد', 'مسیر یادگیری', 'سطوح', 'درخت مهارت', 'دانشکده‌ها', 'دانشنامه', 'آمار', 'دستاوردها', 'پاسپورت من', 'کارنامه']) {
      // exact match: the account chip also links somewhere
      expect(screen.getByRole('link', { name: label })).toBeInTheDocument()
    }
    const shell = container.querySelector('.shell')!
    await userEvent.click(screen.getByRole('button', { name: 'باز کردن منو' }))
    expect(shell).toHaveClass('drawer-open')
    await userEvent.keyboard('{Escape}')
    expect(shell).not.toHaveClass('drawer-open')
  })
})

describe('Authentication', () => {
  const ME = { id: 1, name: 'سارا محمدی', email: 'sara@example.ir', studentId: 'CA-1405-00001', year: 1, levelName: 'جونیور', currentTerm: 1, totalXp: 0, streakDays: 0, joinedAt: '2026-09-28T09:00:00Z' }
  const unauthorized = () => new Response(JSON.stringify({ error: { code: 'unauthorized', message: 'ابتدا وارد حساب کاربری‌ات شو.' } }), { status: 401 })

  it('redirects signed-out visitors from protected pages to login', async () => {
    mockApi({ '/me': unauthorized })
    renderWithProviders(
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/dashboard" element={<RequireAuth><p>داشبورد محرمانه</p></RequireAuth>} />
      </Routes>,
      { route: '/dashboard' },
    )
    expect(await screen.findByRole('heading', { name: 'ورود به کد آکادمی' })).toBeInTheDocument()
    expect(screen.queryByText('داشبورد محرمانه')).not.toBeInTheDocument()
  })

  it('shows the server error for wrong credentials and signs in on success', async () => {
    let attempts = 0
    const fetch = mockApi({
      '/me': unauthorized,
      '/auth/login': () => {
        attempts += 1
        return attempts === 1
          ? new Response(JSON.stringify({ error: { code: 'invalid_credentials', message: 'ایمیل یا رمز عبور نادرست است.' } }), { status: 401 })
          : ME
      },
    })
    renderWithProviders(
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/dashboard" element={<p>خوش آمدی</p>} />
      </Routes>,
      { route: '/login' },
    )
    await userEvent.type(await screen.findByLabelText('ایمیل'), 'sara@example.ir')
    await userEvent.type(screen.getByLabelText('رمز عبور'), 'wrongpass1')
    await userEvent.click(screen.getByRole('button', { name: 'ورود' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('ایمیل یا رمز عبور نادرست است.')
    await userEvent.click(screen.getByRole('button', { name: 'ورود' }))
    expect(await screen.findByText('خوش آمدی')).toBeInTheDocument()
    const body = JSON.parse(String(fetch.mock.calls.find(([u]) => String(u).endsWith('/auth/login'))?.[1]?.body))
    expect(body).toEqual({ email: 'sara@example.ir', password: 'wrongpass1' })
  })

  it('validates matching passwords before registering', async () => {
    const fetch = mockApi({ '/me': unauthorized })
    renderWithProviders(<RegisterPage />, { route: '/register' })
    await userEvent.type(await screen.findByLabelText('نام و نام خانوادگی'), 'سارا محمدی')
    await userEvent.type(screen.getByLabelText('ایمیل'), 'sara@example.ir')
    await userEvent.type(screen.getByLabelText('رمز عبور'), 'tehran1405')
    await userEvent.type(screen.getByLabelText('تکرار رمز عبور'), 'tehran1406')
    await userEvent.click(screen.getByRole('button', { name: 'ثبت‌نام و شروع' }))
    expect(screen.getByRole('alert')).toHaveTextContent('یکسان نیستند')
    expect(fetch.mock.calls.some(([u]) => String(u).includes('/auth/register'))).toBe(false)
  })
})
