import { lazy, Suspense } from 'react'
import { Route, Routes } from 'react-router-dom'
import { Layout } from './components/Layout'
import { RequireAuth } from './components/RequireAuth'
import { LoginPage, RegisterPage } from './pages/AuthPages'
import { Skeleton } from './components/QueryState'

const Home = lazy(() => import('./pages/Home'))
const Dashboard = lazy(() => import('./pages/Dashboard'))
const LearningPath = lazy(() => import('./pages/LearningPath'))
const Levels = lazy(() => import('./pages/Levels'))
const SkillTree = lazy(() => import('./pages/SkillTree'))
const Departments = lazy(() => import('./pages/Departments'))
const DepartmentPage = lazy(() => import('./pages/DepartmentPage'))
const CoursePage = lazy(() => import('./pages/CoursePage'))
const LessonPage = lazy(() => import('./pages/LessonPage'))
const Dictionary = lazy(() => import('./pages/Dictionary'))
const StatsPage = lazy(() => import('./pages/StatsPage'))
const Achievements = lazy(() => import('./pages/Achievements'))
const LeaderboardPage = lazy(() => import('./pages/LeaderboardPage'))
const PassportPage = lazy(() => import('./pages/PassportPage'))
const TranscriptPage = lazy(() => import('./pages/TranscriptPage'))
const NotFound = lazy(() => import('./pages/NotFound'))
const AccountPage = lazy(() => import('./pages/AccountPage'))

/** Public pages; everything else needs a signed-in student. */
const PUBLIC = new Set(['/', '/dictionary', '/login', '/register', '*'])

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        {(
          [
            ['/', Home],
            ['/dashboard', Dashboard],
            ['/path', LearningPath],
            ['/levels', Levels],
            ['/skills', SkillTree],
            ['/departments', Departments],
            ['/departments/:slug', DepartmentPage],
            ['/courses/:slug', CoursePage],
            ['/lessons/:id', LessonPage],
            ['/dictionary', Dictionary],
            ['/stats', StatsPage],
            ['/achievements', Achievements],
            ['/leaderboard', LeaderboardPage],
            ['/passport', PassportPage],
            ['/transcript', TranscriptPage],
            ['/account', AccountPage],
            ['/login', LoginPage],
            ['/register', RegisterPage],
            ['*', NotFound],
          ] as const
        ).map(([path, Page]) => (
          <Route
            key={path}
            path={path}
            element={
              <Suspense fallback={<Skeleton height={160} count={3} />}>
                {PUBLIC.has(path) ? (
                  <Page />
                ) : (
                  <RequireAuth>
                    <Page />
                  </RequireAuth>
                )}
              </Suspense>
            }
          />
        ))}
      </Route>
    </Routes>
  )
}
