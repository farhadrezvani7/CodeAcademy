/** Only allow internal paths as post-login destinations. */
export function safeNext(raw: string | null): string {
  return raw && raw.startsWith('/') && !raw.startsWith('//') ? raw : '/dashboard'
}
