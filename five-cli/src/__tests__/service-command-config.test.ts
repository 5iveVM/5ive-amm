import { describe, expect, it } from '@jest/globals';
import { VmClusterConfigResolver } from '../config/VmClusterConfigResolver.js';

describe('service config hardening', () => {
  it('requires explicit non-placeholder mainnet session service metadata', () => {
    if (process.env.FIVE_ENFORCE_CANONICAL_SERVICE_CONFIG !== '1') {
      return;
    }
    const profile = VmClusterConfigResolver.loadClusterConfig({ cluster: 'mainnet' });
    expect(profile.sessionService).toBeDefined();
    expect(profile.sessionService?.status).toBe('active');
    expect(profile.sessionService?.scriptAccount).not.toBe('11111111111111111111111111111111');
    expect(profile.sessionService?.codeHash).not.toBe('11111111111111111111111111111111');
  });
});
