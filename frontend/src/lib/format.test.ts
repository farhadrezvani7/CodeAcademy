import { faDigits, faDuration, faGrade, faNumber, faPercent, jalali, parseDate, termLabel } from './format'

describe('Persian formatting', () => {
  it('converts digits and decimal point', () => {
    expect(faDigits('سال 3')).toBe('سال ۳')
    expect(faDigits('17.85')).toBe('۱۷٫۸۵')
  })

  it('formats numbers, grades and percents', () => {
    expect(faNumber(21)).toBe('۲۱')
    expect(faGrade(17.85)).toBe('۱۷٫۸۵')
    expect(faGrade(18)).toBe('۱۸٫۰۰')
    expect(faGrade(null)).toBe('—')
    expect(faPercent(75)).toBe('۷۵٪')
  })

  it('formats durations', () => {
    expect(faDuration(35)).toBe('۳۵ دقیقه')
    expect(faDuration(120)).toBe('۲ ساعت')
    expect(faDuration(125)).toBe('۲ ساعت و ۵ دقیقه')
  })

  it('labels terms', () => {
    expect(termLabel(5)).toBe('ترم ۵ · نیمسال اول سال سوم')
    expect(termLabel(12)).toBe('ترم ۱۲ · نیمسال دوم سال ششم')
  })
})

describe('Jalali dates', () => {
  it('parses date-only strings as local dates', () => {
    const d = parseDate('2026-09-28')
    expect([d.getFullYear(), d.getMonth(), d.getDate()]).toEqual([2026, 8, 28])
  })

  it('renders the Persian calendar', () => {
    expect(jalali('2026-09-28')).toBe('۶ مهر ۱۴۰۵')
    expect(jalali('2025-03-21')).toBe('۱ فروردین ۱۴۰۴')
  })
})
