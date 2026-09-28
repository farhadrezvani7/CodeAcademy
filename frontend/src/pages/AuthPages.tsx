import { useId, useState, type FormEvent } from 'react'
import { Link, Navigate, useNavigate, useSearchParams } from 'react-router-dom'
import { useLogin, useRegister, useSession } from '../api/hooks'
import { BrandMark, Icon } from '../components/Icon'
import { safeNext } from '../lib/nav'
import { Skeleton } from '../components/QueryState'

function AuthShell({ title, lead, children }: { title: string; lead: string; children: React.ReactNode }) {
  return (
    <div className="auth-wrap">
      <div className="card auth-card brass-edge">
        <div className="auth-brand">
          <BrandMark className="brand-mark" />
          <div className="brand-name">کد آکادمی</div>
        </div>
        <h1 className="serif">{title}</h1>
        <p className="small text-2" style={{ marginBottom: 18 }}>
          {lead}
        </p>
        {children}
      </div>
    </div>
  )
}

function Field({
  label,
  hint,
  ...input
}: { label: string; hint?: string } & React.InputHTMLAttributes<HTMLInputElement>) {
  const id = useId()
  return (
    <div className="field">
      <label className="field-label" htmlFor={id}>
        {label}
      </label>
      <input className="field-input" id={id} aria-describedby={hint ? `${id}-hint` : undefined} {...input} />
      {hint && (
        <span className="tiny muted" id={`${id}-hint`}>
          {hint}
        </span>
      )}
    </div>
  )
}

function PasswordField({ label, hint, value, onChange, autoComplete }: { label: string; hint?: string; value: string; onChange: (v: string) => void; autoComplete: string }) {
  const [show, setShow] = useState(false)
  const id = useId()
  return (
    <div className="field">
      <label className="field-label" htmlFor={id}>
        {label}
      </label>
      <span className="password-box">
        <input
          id={id}
          aria-describedby={hint ? `${id}-hint` : undefined}
          className="field-input ltr"
          type={show ? 'text' : 'password'}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          autoComplete={autoComplete}
          required
          minLength={8}
          maxLength={128}
          dir="ltr"
        />
        <button type="button" className="password-toggle" onClick={() => setShow((s) => !s)} aria-label={show ? 'پنهان کردن رمز' : 'نمایش رمز'}>
          <Icon name={show ? 'eyeOff' : 'eye'} />
        </button>
      </span>
      {hint && (
        <span className="tiny muted" id={`${id}-hint`}>
          {hint}
        </span>
      )}
    </div>
  )
}

function useSignedInRedirect() {
  const session = useSession()
  const [params] = useSearchParams()
  return { session, next: safeNext(params.get('next')) }
}

export function LoginPage() {
  const { session, next } = useSignedInRedirect()
  const login = useLogin()
  const navigate = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')

  if (session.isPending) return <Skeleton height={320} />
  if (session.data) return <Navigate to={next} replace />

  const submit = (e: FormEvent) => {
    e.preventDefault()
    login.mutate({ email, password }, { onSuccess: () => navigate(next, { replace: true }) })
  }
  return (
    <AuthShell title="ورود به کد آکادمی" lead="با ایمیل و رمز عبورت وارد شو و مسیرت را ادامه بده.">
      <form className="stack" onSubmit={submit} noValidate={false}>
        <Field label="ایمیل" type="email" dir="ltr" className="field-input ltr" value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="email" required maxLength={254} />
        <PasswordField label="رمز عبور" value={password} onChange={setPassword} autoComplete="current-password" />
        {login.isError && (
          <p className="form-error" role="alert">
            {login.error.message}
          </p>
        )}
        <button className="btn primary" type="submit" disabled={login.isPending}>
          {login.isPending ? 'در حال ورود…' : 'ورود'}
        </button>
      </form>
      <p className="small text-2 auth-switch">
        حساب نداری؟ <Link to={`/register${next !== '/dashboard' ? `?next=${encodeURIComponent(next)}` : ''}`}>ثبت‌نام کن</Link>
      </p>
    </AuthShell>
  )
}

export function RegisterPage() {
  const { session, next } = useSignedInRedirect()
  const register = useRegister()
  const navigate = useNavigate()
  const [name, setName] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [localError, setLocalError] = useState<string | null>(null)

  if (session.isPending) return <Skeleton height={420} />
  if (session.data) return <Navigate to={next} replace />

  const submit = (e: FormEvent) => {
    e.preventDefault()
    if (password !== confirm) {
      setLocalError('رمز عبور و تکرار آن یکسان نیستند.')
      return
    }
    setLocalError(null)
    register.mutate({ name, email, password }, { onSuccess: () => navigate('/dashboard', { replace: true }) })
  }
  const error = localError ?? (register.isError ? register.error.message : null)
  return (
    <AuthShell title="ثبت‌نام در سال اول" lead="پرونده‌ی تحصیلی‌ات ساخته می‌شود و از سال اول، جونیور، شروع می‌کنی.">
      <form className="stack" onSubmit={submit}>
        <Field label="نام و نام خانوادگی" value={name} onChange={(e) => setName(e.target.value)} autoComplete="name" required minLength={2} maxLength={60} />
        <Field label="ایمیل" type="email" dir="ltr" className="field-input ltr" value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="email" required maxLength={254} />
        <PasswordField label="رمز عبور" hint="حداقل ۸ نویسه، شامل حرف و عدد" value={password} onChange={setPassword} autoComplete="new-password" />
        <PasswordField label="تکرار رمز عبور" value={confirm} onChange={setConfirm} autoComplete="new-password" />
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
        <button className="btn primary" type="submit" disabled={register.isPending}>
          {register.isPending ? 'در حال ساخت پرونده…' : 'ثبت‌نام و شروع'}
        </button>
      </form>
      <p className="small text-2 auth-switch">
        قبلاً ثبت‌نام کرده‌ای؟ <Link to="/login">وارد شو</Link>
      </p>
    </AuthShell>
  )
}
