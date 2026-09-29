import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';
import { PlanActualComparison } from './PlanActualComparison';
import type { AcquisitionPlan, MediaInspection, VerificationResult } from '../types';

const plan = { operation: { type: 'ENTIRE_MEDIA' } } as AcquisitionPlan;
const inspection = { verificationLevel: 'VERIFIED', containerFormat: 'mp4', fileSizeBytes: 1024 } as MediaInspection;
describe('backend plan verification receipt', () => {
  it('renders backend mismatches without inferring new ones', () => {
    const verification = { planVerification: { conforms: false, warnings: [], mismatches: [
      { field: 'audioCodec', planned: 'aac', actual: null },
    ] } } as unknown as VerificationResult;
    const html = renderToStaticMarkup(<PlanActualComparison plan={plan} inspection={inspection} verification={verification} />);
    expect(html).toContain('audioCodec');
    expect(html).toContain('Missing');
    expect(html).not.toContain('conform to the download plan');
  });
  it('does not claim conformance when verification is missing', () => {
    const html = renderToStaticMarkup(<PlanActualComparison plan={plan} inspection={inspection} verification={null} />);
    expect(html).toContain('has not been verified');
    expect(html).not.toContain('conform to the download plan');
  });
});
