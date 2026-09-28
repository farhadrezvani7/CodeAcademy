import { Link } from 'react-router-dom'
import { useSkillTree } from '../api/hooks'
import type { SkillBranch } from '../api/types'
import { Icon } from '../components/Icon'
import { QueryState } from '../components/QueryState'
import { Grade, NodeBadge, PageHeader, ProgressBar } from '../components/ui'
import { faNumber, faPercent } from '../lib/format'

export default function SkillTree() {
  const query = useSkillTree()
  return (
    <>
      <PageHeader
        eyebrow="چهار شاخه"
        title="درخت مهارت"
        lead="هر گره یک درس زنجیره‌ای با مهارت‌هایی است که به دست می‌آوری. وضعیت گره‌ها مستقیماً از ثبت‌نام‌های تو محاسبه می‌شود."
        actions={
          <div className="row wrap small">
            <NodeBadge state="done" />
            <NodeBadge state="in_progress" />
            <NodeBadge state="locked" />
          </div>
        }
      />
      <QueryState query={query} isEmpty={(b) => b.length === 0}>
        {(branches) => (
          <div className="grid grid-2 tree-grid">
            {branches.map((b) => (
              <Branch key={b.departmentSlug} branch={b} />
            ))}
          </div>
        )}
      </QueryState>
    </>
  )
}

function Branch({ branch }: { branch: SkillBranch }) {
  const titles = new Map(branch.nodes.map((n) => [n.courseSlug, n.code]))
  return (
    <section className={`card branch accent-${branch.accent}`}>
      <div className="row between" style={{ marginBottom: 8 }}>
        <h2 className="serif" style={{ fontSize: '1.15rem' }}>
          {branch.nameFa}
        </h2>
        <span className="tiny muted">
          {faNumber(branch.done)}/{faNumber(branch.total)} · {faPercent(branch.percent)}
        </span>
      </div>
      <ProgressBar value={branch.percent} thin label={branch.nameFa} />
      <ol className="tree">
        {branch.nodes.map((n) => (
          <li key={n.courseSlug} className={`node ${n.state}`}>
            <span className="node-dot" aria-hidden="true">
              {n.state === 'done' ? <Icon name="check" /> : n.state === 'locked' ? <Icon name="lock" /> : <Icon name="play" />}
            </span>
            <Link to={`/courses/${n.courseSlug}`} className="node-body">
              <div className="row between wrap" style={{ gap: 6 }}>
                <span className="row" style={{ gap: 8 }}>
                  <span className="code-tag">{n.code}</span>
                  <strong className="small">{n.title}</strong>
                </span>
                <span className="row" style={{ gap: 6 }}>
                  {n.grade != null && <Grade value={n.grade} />}
                  <NodeBadge state={n.state} />
                </span>
              </div>
              <div className="row wrap" style={{ gap: 6, marginTop: 6 }}>
                {n.skills.map((s) => (
                  <span key={s} className="skill-pill">
                    {s}
                  </span>
                ))}
              </div>
              {n.prerequisites.length > 0 && (
                <div className="tiny muted" style={{ marginTop: 4 }}>
                  پیش‌نیاز: {n.prerequisites.map((p) => titles.get(p) ?? p).join('، ')}
                  {n.prerequisites.some((p) => !titles.has(p)) && ' (از دانشکده‌ی دیگر)'}
                </div>
              )}
            </Link>
          </li>
        ))}
      </ol>
    </section>
  )
}
