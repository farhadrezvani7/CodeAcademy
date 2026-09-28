import { useAchievements } from '../api/hooks'
import { Icon } from '../components/Icon'
import { QueryState } from '../components/QueryState'
import { PageHeader, ProgressBar } from '../components/ui'
import { faNumber, jalali } from '../lib/format'

export default function Achievements() {
  const query = useAchievements()
  return (
    <>
      <PageHeader eyebrow="افتخارات" title="دستاوردها و گواهی‌نامه‌ها" lead="هر دستاورد با یک رفتار واقعی در مسیر یادگیری باز می‌شود؛ گواهی‌نامه‌ها سند رسمی پایان مراحل هستند." />
      <QueryState query={query}>
        {(a) => (
          <>
            <div className="card" style={{ marginBottom: 18 }}>
              <div className="row between small" style={{ marginBottom: 6 }}>
                <span>
                  {faNumber(a.unlocked)} از {faNumber(a.total)} دستاورد باز شده
                </span>
              </div>
              <ProgressBar value={(a.unlocked / Math.max(1, a.total)) * 100} tone="brass" label="دستاوردها" />
            </div>
            <div className="grid grid-4">
              {a.achievements.map((x) => (
                <div key={x.slug} className={`card medal ${x.unlocked ? 'unlocked' : 'locked'}`}>
                  <span className="medal-icon">
                    <Icon name={x.unlocked ? x.icon : 'lock'} />
                  </span>
                  <h3>{x.title}</h3>
                  <p className="tiny text-2">{x.description}</p>
                  <span className={`badge ${x.unlocked ? 'brass' : 'locked'}`}>{x.unlocked ? `بازشده · ${x.unlockedAt ? jalali(x.unlockedAt) : ''}` : 'قفل'}</span>
                </div>
              ))}
            </div>
            <h2 className="section-title">گواهی‌نامه‌ها</h2>
            <div className="grid grid-2">
              {a.certificates.map((c) => (
                <div key={c.slug} className={`certificate ${c.issued ? 'issued' : 'pending'}`}>
                  <div className="cert-inner">
                    <div className="cert-seal">
                      <Icon name="medal" />
                    </div>
                    <div className="cert-org serif">کد آکادمی</div>
                    <h3 className="serif">{c.title}</h3>
                    <p className="small">{c.description}</p>
                    {c.issued ? (
                      <div className="cert-meta tiny">
                        <span>تاریخ صدور: {c.issuedAt && jalali(c.issuedAt)}</span>
                        <span className="ltr">{c.serialNo}</span>
                      </div>
                    ) : (
                      <div className="cert-meta tiny">
                        <span>
                          <Icon name="lock" size={13} /> هنوز صادر نشده
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </>
        )}
      </QueryState>
    </>
  )
}
