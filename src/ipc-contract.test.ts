import { expect, it } from 'vitest';
import type { AcquisitionOperation, AcquisitionPlan, PlannedTransform } from './types';

it('uses JSON numbers consistently for operation, plan and transform milliseconds', () => {
  const operation: AcquisitionOperation = { type: 'CLIP', startMs: 1250, endMs: 9750 };
  const transform: PlannedTransform = { type: 'TRIM', startMs: operation.startMs, endMs: operation.endMs };
  const timeRangeMs: AcquisitionPlan['timeRangeMs'] = [transform.startMs, transform.endMs];
  expect(JSON.parse(JSON.stringify(transform))).toEqual({ type: 'TRIM', startMs: 1250, endMs: 9750 });
  expect(timeRangeMs).toEqual([1250, 9750]);
});
