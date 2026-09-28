// Response shapes of the Code Academy API (/api/v1). Mirrors the Rust DTOs.

export type EnrollmentStatus = 'locked' | 'enrolled' | 'in_progress' | 'done'
export type NodeState = 'done' | 'in_progress' | 'locked'
export type StageState = 'passed' | 'current' | 'upcoming'
export type CourseKind = 'course' | 'project'
export type Accent = 'brass' | 'mint' | 'sky' | 'rose'

export interface CourseLink {
  slug: string
  code: string
  title: string
  done: boolean
}

export interface CourseCard {
  id: number
  slug: string
  code: string
  title: string
  summary: string
  credits: number
  kind: CourseKind
  year: number
  termInYear: number
  levelName: string
  departmentSlug: string
  departmentName: string
  status: EnrollmentStatus
  grade: number | null
  passed: boolean
  needsRetake: boolean
  lessonsDone: number
  lessonCount: number
  percent: number
  prerequisites: CourseLink[]
  skills: string[]
}

export interface LessonItem {
  id: number
  order: number
  title: string
  estimatedMinutes: number
  completed: boolean
}

export interface CourseDetail extends CourseCard {
  lessons: LessonItem[]
  nextLessonId: number | null
}

export interface RuleDefinition {
  key: string
  title: string
  description: string
}

export interface RuleResult {
  key: string
  title: string
  met: boolean
  blocking: string[]
}

export interface PromotionEvaluation {
  year: number
  eligible: boolean
  rules: RuleResult[]
}

export interface Level {
  year: number
  slug: string
  nameFa: string
  nameEn: string
  yearBlurb: string
  description: string
  requiredCredits: number
  courseCount: number
  courseCountLabel: string
  project: CourseLink | null
  state: StageState
  creditsPassed: number
  percent: number
  courses: CourseLink[]
  evaluation: PromotionEvaluation | null
}

export interface LevelsResponse {
  currentYear: number
  rules: RuleDefinition[]
  levels: Level[]
}

export interface LearningPath {
  currentYear: number
  currentTerm: number
  graduated: boolean
  creditsPassed: number
  totalCredits: number
  percent: number
  stages: Level[]
}

export interface Department {
  slug: string
  nameFa: string
  nameEn: string
  codePrefix: string
  accent: Accent
  description: string
  courseCount: number
  projectCount: number
  credits: number
  creditsPassed: number
  percent: number
}

export interface DepartmentDetail extends Department {
  courses: CourseDetail[]
}

export interface SkillNode {
  courseSlug: string
  code: string
  title: string
  year: number
  state: NodeState
  grade: number | null
  skills: string[]
  prerequisites: string[]
}

export interface SkillBranch {
  departmentSlug: string
  nameFa: string
  accent: Accent
  done: number
  total: number
  percent: number
  nodes: SkillNode[]
}

export interface CodeExample {
  language: string
  code: string
  explanation: string
}

export interface TermRef {
  slug: string
  nameEn: string
  nameFa: string
}

export interface Lesson {
  id: number
  order: number
  title: string
  estimatedMinutes: number
  what: string
  why: string
  simpleExample: CodeExample
  realExample: CodeExample
  mistakes: string[]
  bestPractices: string[]
  relatedTerms: TermRef[]
  tryIt: string
  course: {
    slug: string
    code: string
    title: string
    departmentSlug: string
    departmentName: string
    status: EnrollmentStatus
  }
  lessonCount: number
  previous: { id: number; title: string } | null
  next: { id: number; title: string } | null
  locked: boolean
  completed: boolean
  selfScore: number | null
  scoreOptions: { score: number; label: string }[]
}

export interface CompleteLessonResult {
  firstCompletion: boolean
  xpGained: number
  totalXp: number
  courseStatus: EnrollmentStatus
  courseGrade: number | null
  unlockedCourses: string[]
  promotedTo: string | null
  graduated: boolean
  streakDays: number
  missionCompleted: boolean
  newAchievements: string[]
  newCertificates: string[]
}

export interface Term {
  slug: string
  nameEn: string
  nameFa: string
  category: string
  categoryName: string
  definition: string
  example: string
  related: TermRef[]
}

export interface Achievement {
  slug: string
  title: string
  description: string
  icon: string
  unlocked: boolean
  unlockedAt: string | null
}

export interface Certificate {
  slug: string
  title: string
  description: string
  issued: boolean
  serialNo: string | null
  issuedAt: string | null
}

export interface AchievementsResponse {
  unlocked: number
  total: number
  achievements: Achievement[]
  certificates: Certificate[]
}

export interface DayMinutes {
  date: string
  minutes: number
}

export interface Stats {
  today: string
  heatmap: DayMinutes[]
  weekly: DayMinutes[]
  byTopic: { slug: string; nameFa: string; accent: Accent; minutes: number; percent: number }[]
  totals: {
    minutes: number
    activeDays: number
    averagePerActiveDay: number
    currentStreak: number
    longestStreak: number
    lessonsCompleted: number
  }
}

export interface TranscriptCourse {
  slug: string
  code: string
  title: string
  kind: CourseKind
  credits: number
  grade: number | null
  passed: boolean
}

export interface Transcript {
  studentName: string
  studentId: string
  levelName: string
  year: number
  currentTerm: number
  semesters: { number: number; title: string; courses: TranscriptCourse[]; credits: number; creditsPassed: number; gpa: number | null }[]
  inProgress: { slug: string; code: string; title: string; credits: number; status: EnrollmentStatus; percent: number }[]
  cumulativeGpa: number | null
  creditsAttempted: number
  creditsPassed: number
  creditsRemaining: number
  totalCredits: number
  issuedAt: string
}

export interface RankInfo {
  rank: number
  of: number
  levelName: string
}

export interface Passport {
  name: string
  studentId: string
  joinedAt: string
  levelName: string
  levelNameEn: string
  year: number
  currentTerm: number
  totalXp: number
  streakDays: number
  gpa: number | null
  graduated: boolean
  graduatedAt: string | null
  stamps: { year: number; nameFa: string; nameEn: string; passed: boolean; gpa: number | null; passedAt: string | null }[]
  skills: { departmentSlug: string; nameFa: string; accent: Accent; percent: number; creditsPassed: number; credits: number }[]
  rank: RankInfo
  certificates: Certificate[]
  achievementsUnlocked: number
  achievementsTotal: number
}

export interface Leaderboard extends RankInfo {
  entries: { rank: number; name: string; xp: number; isMe: boolean }[]
}

export interface Mission {
  id: number
  date: string
  description: string
  courseSlug: string | null
  courseTitle: string | null
  targetMinutes: number
  progressMinutes: number
  percent: number
  xpReward: number
  completed: boolean
}

export type SuggestionKind = 'review_concept' | 'study_lesson' | 'study_and_practice'

export interface Suggestion {
  kind: SuggestionKind
  title: string
  description: string
  lessonId: number | null
  termSlug: string | null
}

export interface CheckInResponse {
  minutes: number
  suggestion: Suggestion
}

export interface Dashboard {
  today: string
  student: {
    name: string
    studentId: string
    year: number
    currentTerm: number
    levelName: string
    levelNameEn: string
    levelBlurb: string
    totalXp: number
    streakDays: number
    longestStreak: number
    gpa: number | null
    graduated: boolean
  }
  progress: {
    overallPercent: number
    creditsPassed: number
    totalCredits: number
    yearPercent: number
    yearCreditsPassed: number
    yearCredits: number
  }
  currentCourse: CourseDetail | null
  mission: Mission
  checkIn: CheckInResponse | null
  project: {
    slug: string
    code: string
    title: string
    status: EnrollmentStatus
    percent: number
    lessonsDone: number
    lessonCount: number
    grade: number | null
  } | null
  promotion: PromotionEvaluation
  recentAchievements: Achievement[]
  recentActivity: { date: string; minutes: number; topic: string; source: 'lesson' | 'study'; courseTitle: string | null; at: string }[]
  skills: { departmentSlug: string; nameFa: string; accent: Accent; percent: number; openCourses: number }[]
  advisor: { name: string; title: string; advice: { message: string; focusDepartment: string | null; tone: string } } | null
  announcements: { id: number; title: string; body: string; important: boolean; publishedAt: string }[]
  rank: RankInfo
}

export interface Me {
  id: number
  name: string
  email: string
  studentId: string
  year: number
  levelName: string
  currentTerm: number
  totalXp: number
  streakDays: number
  joinedAt: string
}

export interface Highlight {
  section: string
  icon: string
  title: string
  body: string
}

export interface Overview {
  highlights: Highlight[]
  stats: { years: number; departments: number; courses: number; projects: number; lessons: number; credits: number; terms: number; students: number }
  levels: { year: number; nameFa: string; nameEn: string; yearBlurb: string; requiredCredits: number; courseCountLabel: string }[]
  departments: { slug: string; nameFa: string; nameEn: string; accent: Accent; description: string; courseCount: number }[]
}

export interface StudyOutcome {
  streakDays: number
  missionCompletedNow: boolean
  newAchievements: string[]
  newCertificates: string[]
}
