import { useEffect, useState } from 'react'
import { NavLink, Outlet, useLocation } from 'react-router-dom'
import { useSession } from '../api/hooks'
import { faNumber } from '../lib/format'
import { useTheme } from '../lib/theme'
import { BrandMark, Icon } from './Icon'

const NAV: { section: string; items: { to: string; label: string; icon: string }[] }[] = [
  {
    section: 'آکادمی',
    items: [
      { to: '/', label: 'خانه', icon: 'home' },
      { to: '/dashboard', label: 'داشبورد', icon: 'dashboard' },
      { to: '/path', label: 'مسیر یادگیری', icon: 'path' },
      { to: '/levels', label: 'سطوح', icon: 'levels' },
      { to: '/skills', label: 'درخت مهارت', icon: 'tree' },
      { to: '/departments', label: 'دانشکده‌ها', icon: 'building' },
      { to: '/dictionary', label: 'دانشنامه', icon: 'book' },
    ],
  },
  {
    section: 'پرونده‌ی من',
    items: [
      { to: '/stats', label: 'آمار', icon: 'chart' },
      { to: '/achievements', label: 'دستاوردها', icon: 'trophy' },
      { to: '/leaderboard', label: 'رتبه‌بندی', icon: 'users' },
      { to: '/passport', label: 'پاسپورت من', icon: 'passport' },
      { to: '/transcript', label: 'کارنامه', icon: 'transcript' },
      { to: '/account', label: 'حساب کاربری', icon: 'settings' },
    ],
  },
]

function Brand() {
  return (
    <NavLink to="/" className="brand" aria-label="کد آکادمی — خانه">
      <BrandMark className="brand-mark" />
      <div>
        <div className="brand-name">کد آکادمی</div>
        <div className="brand-sub">CODE ACADEMY</div>
      </div>
    </NavLink>
  )
}

function ThemeButton() {
  const { theme, toggle } = useTheme()
  const next = theme === 'dark' ? 'روشن' : 'تیره'
  return (
    <button className="icon-btn" onClick={toggle} aria-label={`تغییر به حالت ${next}`} title={`حالت ${next}`}>
      <Icon name={theme === 'dark' ? 'sun' : 'moon'} />
    </button>
  )
}

function StudentChip() {
  const me = useSession()
  if (me.isPending) return <div className="skeleton" style={{ height: 58 }} />
  if (!me.data) {
    return (
      <div className="stack" style={{ gap: 8 }}>
        <NavLink to="/register" className="btn primary">
          ثبت‌نام
        </NavLink>
        <NavLink to="/login" className="btn ghost">
          <Icon name="login" /> ورود
        </NavLink>
      </div>
    )
  }
  return (
    <NavLink to="/account" className="student-chip" aria-label="حساب کاربری">
      <span className="avatar">{me.data.name.charAt(0)}</span>
      <span style={{ minWidth: 0 }}>
        <div className="name">{me.data.name}</div>
        <div className="meta">
          {me.data.levelName} · {faNumber(me.data.totalXp)} XP
        </div>
      </span>
    </NavLink>
  )
}

export function Layout() {
  const [open, setOpen] = useState(false)
  const location = useLocation()

  useEffect(() => {
    window.scrollTo({ top: 0 })
  }, [location.pathname])

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && setOpen(false)
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [open])

  return (
    <div className={`shell ${open ? 'drawer-open' : ''}`}>
      <header className="topbar">
        <button className="icon-btn" onClick={() => setOpen(true)} aria-label="باز کردن منو" aria-expanded={open} aria-controls="sidebar">
          <Icon name="menu" />
        </button>
        <Brand />
        <ThemeButton />
      </header>

      <aside className="sidebar" id="sidebar" aria-label="ناوبری اصلی">
        <div className="row between">
          <Brand />
        </div>
        <nav className="nav">
          {NAV.map((group) => (
            <div key={group.section} className="nav">
              <div className="nav-section">{group.section}</div>
              {group.items.map((item) => (
                <NavLink key={item.to} to={item.to} end={item.to === '/'} onClick={() => setOpen(false)}>
                  <Icon name={item.icon} />
                  {item.label}
                </NavLink>
              ))}
            </div>
          ))}
        </nav>
        <div className="sidebar-foot">
          <StudentChip />
          <div className="row between">
            <span className="tiny muted">حالت نمایش</span>
            <ThemeButton />
          </div>
        </div>
      </aside>
      <button className="drawer-backdrop" aria-label="بستن منو" onClick={() => setOpen(false)} tabIndex={open ? 0 : -1} />

      <main className="main">
        <div className="content">
          <Outlet />
        </div>
      </main>
    </div>
  )
}
