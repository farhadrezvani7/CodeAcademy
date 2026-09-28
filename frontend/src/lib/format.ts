// Presentation helpers: Persian digits and Jalali dates.

const FA_DIGITS = '۰۱۲۳۴۵۶۷۸۹'

/** Replaces ASCII digits with Persian digits (and `.` between digits with `٫`). */
export function faDigits(input: string | number): string {
  return String(input)
    .replace(/(\d)\.(\d)/g, '$1٫$2')
    .replace(/\d/g, (d) => FA_DIGITS[Number(d)])
}

const intFmt = new Intl.NumberFormat('fa-IR')

/** 1234 → ۱٬۲۳۴ */
export function faNumber(n: number): string {
  return intFmt.format(n)
}

/** Grades and GPAs: always two decimals, e.g. 17.5 → ۱۷٫۵۰ */
export function faGrade(n: number | null | undefined): string {
  if (n === null || n === undefined) return '—'
  return new Intl.NumberFormat('fa-IR', { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(n)
}

export function faPercent(n: number): string {
  return `${faNumber(Math.round(n))}٪`
}

/** Parses `YYYY-MM-DD` as a local calendar date (no timezone shift). */
export function parseDate(value: string): Date {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value)
  return m ? new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3])) : new Date(value)
}

const TZ = 'Asia/Tehran'
const jalaliLong = new Intl.DateTimeFormat('fa-IR-u-ca-persian', { year: 'numeric', month: 'long', day: 'numeric' })
const jalaliLongTz = new Intl.DateTimeFormat('fa-IR-u-ca-persian', { year: 'numeric', month: 'long', day: 'numeric', timeZone: TZ })
const jalaliShort = new Intl.DateTimeFormat('fa-IR-u-ca-persian', { month: 'long', day: 'numeric' })
const jalaliMonth = new Intl.DateTimeFormat('fa-IR-u-ca-persian', { month: 'long' })
const jalaliWeekday = new Intl.DateTimeFormat('fa-IR-u-ca-persian', { weekday: 'short' })
const jalaliYearMonth = new Intl.DateTimeFormat('fa-IR-u-ca-persian', { year: 'numeric', month: 'long' })
const timeFmt = new Intl.DateTimeFormat('fa-IR', { hour: '2-digit', minute: '2-digit', timeZone: TZ })

const toDate = (v: string | Date) => (typeof v === 'string' ? parseDate(v) : v)

const isDateOnly = (v: string | Date) => typeof v === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(v)

/**
 * ۶ مهر ۱۴۰۵. Calendar dates (`YYYY-MM-DD`) are shown as-is; timestamps are
 * shown in Tehran time so the day never shifts with the browser's timezone.
 */
export const jalali = (v: string | Date) => (isDateOnly(v) ? jalaliLong.format(toDate(v)) : jalaliLongTz.format(toDate(v)))
/** ۶ مهر */
export const jalaliDayMonth = (v: string | Date) => jalaliShort.format(toDate(v))
export const jalaliMonthName = (v: string | Date) => jalaliMonth.format(toDate(v))
export const jalaliYearMonthName = (v: string | Date) => jalaliYearMonth.format(toDate(v))
export const weekdayShort = (v: string | Date) => jalaliWeekday.format(toDate(v))
export const clockTime = (v: string | Date) => timeFmt.format(toDate(v))

/** "۳۵ دقیقه" or "۲ ساعت و ۵ دقیقه" */
export function faDuration(minutes: number): string {
  const h = Math.floor(minutes / 60)
  const m = minutes % 60
  if (h === 0) return `${faNumber(m)} دقیقه`
  if (m === 0) return `${faNumber(h)} ساعت`
  return `${faNumber(h)} ساعت و ${faNumber(m)} دقیقه`
}

const ORDINALS = ['اول', 'دوم', 'سوم', 'چهارم', 'پنجم', 'ششم', 'هفتم', 'هشتم', 'نهم', 'دهم', 'یازدهم', 'دوازدهم']
export const faOrdinal = (n: number) => ORDINALS[n - 1] ?? faNumber(n)

/** Term 5 → "ترم ۵ (نیمسال اول سال سوم)" */
export function termLabel(term: number): string {
  const year = Math.ceil(term / 2)
  const half = term % 2 === 1 ? 'اول' : 'دوم'
  return `ترم ${faNumber(term)} · نیمسال ${half} سال ${faOrdinal(year)}`
}
