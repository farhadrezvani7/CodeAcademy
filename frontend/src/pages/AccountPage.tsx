import { useState, type FormEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import { useChangePassword, useLogout, useSession, useUpdateProfile } from '../api/hooks'
import { Icon } from '../components/Icon'
import { useToast } from '../components/Toast'
import { Card, PageHeader } from '../components/ui'
import { jalali } from '../lib/format'

export default function AccountPage() {
  const session = useSession()
  const me = session.data
  const update = useUpdateProfile()
  const change = useChangePassword()
  const logout = useLogout()
  const toast = useToast()
  const navigate = useNavigate()
  const [name, setName] = useState(me?.name ?? '')
  const [current, setCurrent] = useState('')
  const [next, setNext] = useState('')
  const [confirm, setConfirm] = useState('')
  const [pwError, setPwError] = useState<string | null>(null)

  if (!me) return null

  const saveName = (e: FormEvent) => {
    e.preventDefault()
    update.mutate(name, { onSuccess: () => toast('نام ذخیره شد.') })
  }
  const savePassword = (e: FormEvent) => {
    e.preventDefault()
    if (next !== confirm) return setPwError('رمز جدید و تکرار آن یکسان نیستند.')
    setPwError(null)
    change.mutate(
      { currentPassword: current, newPassword: next },
      {
        onSuccess: () => {
          setCurrent('')
          setNext('')
          setConfirm('')
          toast('رمز عبور عوض شد و از بقیه‌ی دستگاه‌ها خارج شدی.')
        },
      },
    )
  }

  return (
    <>
      <PageHeader eyebrow="تنظیمات" title="حساب کاربری" lead={`شماره‌ی دانشجویی ${me.studentId} · عضو از ${jalali(me.joinedAt)}`} />
      <div className="grid grid-2">
        <Card title="مشخصات" icon="user">
          <form className="stack" onSubmit={saveName}>
            <label className="field">
              <span className="field-label">نام و نام خانوادگی</span>
              <input className="field-input" value={name} onChange={(e) => setName(e.target.value)} required minLength={2} maxLength={60} autoComplete="name" />
            </label>
            <label className="field">
              <span className="field-label">ایمیل</span>
              <input className="field-input ltr" value={me.email} dir="ltr" readOnly disabled />
            </label>
            {update.isError && <p className="form-error" role="alert">{update.error.message}</p>}
            <button className="btn primary" type="submit" disabled={update.isPending || name.trim() === me.name}>
              ذخیره
            </button>
          </form>
        </Card>
        <Card title="تغییر رمز عبور" icon="lock">
          <form className="stack" onSubmit={savePassword}>
            {(
              [
                ['رمز فعلی', current, setCurrent, 'current-password'],
                ['رمز جدید (حداقل ۸ نویسه، حرف و عدد)', next, setNext, 'new-password'],
                ['تکرار رمز جدید', confirm, setConfirm, 'new-password'],
              ] as const
            ).map(([label, value, set, ac]) => (
              <label key={label} className="field">
                <span className="field-label">{label}</span>
                <input className="field-input ltr" dir="ltr" type="password" value={value} onChange={(e) => set(e.target.value)} autoComplete={ac} required minLength={8} maxLength={128} />
              </label>
            ))}
            {(pwError || change.isError) && <p className="form-error" role="alert">{pwError ?? change.error?.message}</p>}
            <button className="btn primary" type="submit" disabled={change.isPending}>
              تغییر رمز
            </button>
          </form>
        </Card>
      </div>
      <div className="card" style={{ marginTop: 18 }}>
        <div className="row between wrap">
          <div>
            <strong>خروج از حساب</strong>
            <p className="small text-2">روی این دستگاه از حسابت خارج می‌شوی.</p>
          </div>
          <button className="btn danger" onClick={() => logout.mutate(undefined, { onSettled: () => navigate('/', { replace: true }) })} disabled={logout.isPending}>
            <Icon name="logout" /> خروج
          </button>
        </div>
      </div>
    </>
  )
}
