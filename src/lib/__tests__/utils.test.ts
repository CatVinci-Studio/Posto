import { describe, it, expect } from 'vitest'
import { cn, formatRelativeTime } from '../utils'

describe('cn', () => {
  it('merges class names', () => {
    expect(cn('a', 'b')).toBe('a b')
  })

  it('skips falsy values', () => {
    expect(cn('a', false, null, undefined, 'b')).toBe('a b')
  })

  it('lets later tailwind utilities win', () => {
    expect(cn('p-2', 'p-4')).toBe('p-4')
  })
})

describe('formatRelativeTime', () => {
  it('returns a non-empty string for sub-minute timestamps', () => {
    // Intl.RelativeTimeFormat output for `format(0, 'minute')` varies by
    // locale and engine version (e.g. "now", "this minute"). Just verify
    // the helper produces something.
    const result = formatRelativeTime(Date.now(), 'en-US')
    expect(result.length).toBeGreaterThan(0)
  })

  it('falls back to a locale date for timestamps older than a week', () => {
    const eightDaysAgo = Date.now() - 8 * 24 * 60 * 60 * 1000
    const result = formatRelativeTime(eightDaysAgo, 'en-US')
    // en-US locale date contains slashes (M/D/YYYY)
    expect(result).toMatch(/\//)
  })
})
