const SECRET_HEADER = /\b(authorization|proxy-authorization|cookie|set-cookie|x-api-key)\s*[:=]\s*[^\r\n]*/gi;
const HTTP_URL = /https?:\/\/[^\s"'<>]+/gi;

export function sanitizeTechnicalError(value: unknown): string {
  const message = value instanceof Error ? value.message : String(value);
  return message
    .replace(SECRET_HEADER, '$1: [REDACTED]')
    .replace(HTTP_URL, (rawUrl) => {
      try {
        const url = new URL(rawUrl);
        return `${url.origin}/[REDACTED]`;
      } catch {
        return '[REDACTED URL]';
      }
    });
}
