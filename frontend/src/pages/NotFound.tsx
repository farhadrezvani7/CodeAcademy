import { Link } from 'react-router-dom'
import { Icon } from '../components/Icon'

export default function NotFound() {
  return (
    <div className="state-box" style={{ marginTop: 40 }}>
      <Icon name="search" />
      <h3>این صفحه پیدا نشد</h3>
      <p className="small">شاید آدرس اشتباه است یا صفحه جابه‌جا شده.</p>
      <Link to="/dashboard" className="btn primary sm">
        بازگشت به داشبورد
      </Link>
    </div>
  )
}
