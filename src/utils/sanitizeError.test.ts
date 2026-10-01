import { describe, expect, it } from 'vitest';
import { sanitizeTechnicalError } from './sanitizeError';

describe('sanitizeTechnicalError', () => {
  it('removes URL paths, credentials, queries, fragments, and secret headers', () => {
    const result = sanitizeTechnicalError(
      'Request https://user:pass@example.com/private?token=secret#fragment failed; Authorization: Bearer secret',
    );
    expect(result).toBe('Request https://example.com/[REDACTED] failed; Authorization: [REDACTED]');
    expect(result).not.toContain('secret');
    expect(result).not.toContain('private');
  });
});
