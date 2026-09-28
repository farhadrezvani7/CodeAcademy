import { useLeaderboard } from '../api/hooks'
import { QueryState } from '../components/QueryState'
import { PageHeader } from '../components/ui'
import { faNumber } from '../lib/format'

export default function LeaderboardPage() {
  const query = useLeaderboard()
  return (
    <>
      <PageHeader eyebrow="هم‌دوره‌ای‌ها" title="رتبه‌بندی" lead="جایگاه تو بین دانشجویان هم‌سطح، بر اساس XP." />
      <QueryState query={query} isEmpty={(b) => b.entries.length === 0}>
        {(b) => (
          <div className="card" style={{ maxWidth: 720 }}>
            <div className="row between" style={{ marginBottom: 12 }}>
              <strong className="serif">سطح {b.levelName}</strong>
              <span className="badge brass">
                رتبه‌ی تو: {faNumber(b.rank)} از {faNumber(b.of)}
              </span>
            </div>
            <ol className="leaderboard">
              {b.entries.map((e, i) => (
                <li key={i} className={e.isMe ? 'me' : ''}>
                  <span className={`rank-no r${e.rank}`}>{faNumber(e.rank)}</span>
                  <span className="avatar">{e.name.charAt(0)}</span>
                  <span className="leader-name">
                    {e.name} {e.isMe && <span className="badge enrolled">تو</span>}
                  </span>
                  <strong>
                    {faNumber(e.xp)} <span className="tiny muted">XP</span>
                  </strong>
                </li>
              ))}
            </ol>
            {b.of === 1 && <p className="small muted" style={{ marginTop: 12 }}>فعلاً تنها دانشجوی این سطح هستی؛ با ورود هم‌سطح‌ها، رتبه‌بندی کامل‌تر می‌شود.</p>}
          </div>
        )}
      </QueryState>
    </>
  )
}
