jest.mock('chalk', () => ({
  __esModule: true,
  default: {
    hex: () => (s: string) => s
  }
}));

import { brandLine, section, success, error, hint, commandNotFound } from '../cli-ui.js';

describe('cli-ui helpers', () => {
  it('renders brand line and section labels', () => {
    expect(brandLine()).toContain('5IVE CLI');
    expect(section('Quick Start')).toBe('QUICK START');
  });

  it('formats status lines', () => {
    expect(success('Done')).toContain('OK');
    expect(error('Nope')).toContain('error:');
    expect(hint('Try again')).toContain('hint:');
  });

  it('formats command not found with suggestions', () => {
    const output = commandNotFound('buidl', ['build', 'compile']);
    expect(output).toContain('unknown command');
    expect(output).toContain('build');
  });
});
