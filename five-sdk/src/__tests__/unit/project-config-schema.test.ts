import { parseProjectConfig } from '../../project/config.js';

describe('parseProjectConfig schema/dependency validation', () => {
  it('parses schema_version and alias dependencies', () => {
    const config = parseProjectConfig({
      schema_version: 1,
      project: {
        name: 'demo',
        version: '0.1.0',
        source_dir: 'src',
        build_dir: 'build',
        entry_point: 'src/main.v',
        target: 'vm',
      },
      dependencies: {
        std: {
          package: '@5ive/std',
          version: '0.1.0',
          source: 'bundled',
          link: 'inline',
        },
      },
    });

    expect(config.schemaVersion).toBe(1);
    expect(config.dependencies).toEqual([
      {
        alias: 'std',
        package: '@5ive/std',
        version: '0.1.0',
        source: 'bundled',
        link: 'inline',
        path: undefined,
        namespace: undefined,
        address: undefined,
        moatAccount: undefined,
        module: undefined,
        pin: undefined,
        cluster: undefined,
      },
    ]);
  });

  it('fails when schema_version is missing', () => {
    expect(() =>
      parseProjectConfig({
        project: { name: 'demo', version: '0.1.0' },
      }),
    ).toThrow("Missing required top-level 'schema_version'");
  });

  it('fails for invalid moat dependency shape', () => {
    expect(() =>
      parseProjectConfig({
        schema_version: 1,
        project: { name: 'demo', version: '0.1.0' },
        dependencies: {
          risk: {
            package: '@acme/risk',
            source: 'moat',
            link: 'external',
            moat_account: 'MoAt111111111111111111111111111111111111111',
          },
        },
      }),
    ).toThrow("source=moat requires 'moat_account' and 'module'");
  });
});
