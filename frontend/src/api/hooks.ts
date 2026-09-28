import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api, isUnauthorized } from './client'
import type * as T from './types'

const get =
  <R>(path: string) =>
  ({ signal }: { signal: AbortSignal }) =>
    api.get<R>(path, signal)

export const useOverview = () => useQuery({ queryKey: ['overview'], queryFn: get<T.Overview>('/overview') })
/** Current student, or `null` when signed out. */
export const useSession = () =>
  useQuery({
    queryKey: ['me'],
    queryFn: async ({ signal }) => {
      try {
        return await api.get<T.Me>('/me', signal)
      } catch (e) {
        if (isUnauthorized(e)) return null
        throw e
      }
    },
    staleTime: 60_000,
  })
export const useDashboard = () => useQuery({ queryKey: ['dashboard'], queryFn: get<T.Dashboard>('/dashboard') })
export const useLevels = () => useQuery({ queryKey: ['levels'], queryFn: get<T.LevelsResponse>('/levels') })
export const useLearningPath = () => useQuery({ queryKey: ['learning-path'], queryFn: get<T.LearningPath>('/learning-path') })
export const useSkillTree = () => useQuery({ queryKey: ['skill-tree'], queryFn: get<T.SkillBranch[]>('/skill-tree') })
export const useDepartments = () => useQuery({ queryKey: ['departments'], queryFn: get<T.Department[]>('/departments') })
export const useDepartment = (slug: string) =>
  useQuery({ queryKey: ['department', slug], queryFn: get<T.DepartmentDetail>(`/departments/${encodeURIComponent(slug)}`) })
export const useCourse = (slug: string) =>
  useQuery({ queryKey: ['course', slug], queryFn: get<T.CourseDetail>(`/courses/${encodeURIComponent(slug)}`) })
export const useLesson = (id: string) => useQuery({ queryKey: ['lesson', id], queryFn: get<T.Lesson>(`/lessons/${encodeURIComponent(id)}`) })
export const useAchievements = () => useQuery({ queryKey: ['achievements'], queryFn: get<T.AchievementsResponse>('/achievements') })
export const useStats = () => useQuery({ queryKey: ['stats'], queryFn: get<T.Stats>('/stats') })
export const useTranscript = () => useQuery({ queryKey: ['transcript'], queryFn: get<T.Transcript>('/transcript') })
export const usePassport = () => useQuery({ queryKey: ['passport'], queryFn: get<T.Passport>('/passport') })
export const useLeaderboard = () => useQuery({ queryKey: ['leaderboard'], queryFn: get<T.Leaderboard>('/leaderboard') })

export const useDictionary = (q: string) =>
  useQuery({
    queryKey: ['dictionary', q],
    queryFn: get<T.Term[]>(q ? `/dictionary?q=${encodeURIComponent(q)}` : '/dictionary'),
    placeholderData: keepPreviousData,
  })

/** Everything derived from student progress; refreshed after any write. */
const PROGRESS_KEYS = [
  'me', 'dashboard', 'levels', 'learning-path', 'skill-tree', 'departments', 'department',
  'course', 'lesson', 'achievements', 'stats', 'transcript', 'passport', 'leaderboard',
]

function useInvalidateProgress() {
  const qc = useQueryClient()
  return () => Promise.all(PROGRESS_KEYS.map((key) => qc.invalidateQueries({ queryKey: [key] })))
}

export function useCompleteLesson(lessonId: number) {
  const invalidate = useInvalidateProgress()
  return useMutation({
    mutationFn: (selfScore: number) => api.post<T.CompleteLessonResult>(`/lessons/${lessonId}/complete`, { selfScore }),
    onSuccess: invalidate,
  })
}

export function useCheckIn() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (minutes: number) => api.post<T.CheckInResponse>('/check-in', { minutes }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['dashboard'] }),
  })
}

export function useLogStudy() {
  const invalidate = useInvalidateProgress()
  return useMutation({
    mutationFn: (body: { minutes: number; courseSlug?: string | null }) => api.post<T.StudyOutcome>('/activity', body),
    onSuccess: invalidate,
  })
}

function useAfterSignIn() {
  const qc = useQueryClient()
  return async (me: T.Me) => {
    qc.removeQueries({ predicate: (q) => q.queryKey[0] !== 'me' })
    qc.setQueryData(['me'], me)
  }
}

export function useLogin() {
  const after = useAfterSignIn()
  return useMutation({
    mutationFn: (body: { email: string; password: string }) => api.post<T.Me>('/auth/login', body),
    onSuccess: after,
  })
}

export function useRegister() {
  const after = useAfterSignIn()
  return useMutation({
    mutationFn: (body: { name: string; email: string; password: string }) => api.post<T.Me>('/auth/register', body),
    onSuccess: after,
  })
}

export function useLogout() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: () => api.post<null>('/auth/logout'),
    onSettled: () => {
      qc.clear()
      qc.setQueryData(['me'], null)
    },
  })
}

export function useUpdateProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (name: string) => api.put<T.Me>('/me', { name }),
    onSuccess: (me) => {
      qc.setQueryData(['me'], me)
      qc.invalidateQueries({ predicate: (q) => q.queryKey[0] !== 'me' })
    },
  })
}

export function useChangePassword() {
  return useMutation({
    mutationFn: (body: { currentPassword: string; newPassword: string }) => api.post<null>('/me/password', body),
  })
}
