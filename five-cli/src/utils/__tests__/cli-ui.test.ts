jest.mock('chalk', () => {
  const mockColorFunction = (s: string) => s;
  return {
    __esModule: true,
    default: {
      hex: () => mockColorFunction,
      white: mockColorFunction,
      gray: mockColorFunction,
      magenta: mockColorFunction,
      magentaBright: mockColorFunction,
      green: mockColorFunction,
      yellow: mockColorFunction,
      red: mockColorFunction,
      cyan: mockColorFunction,
      bold: mockColorFunction
    }
  };
});

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
