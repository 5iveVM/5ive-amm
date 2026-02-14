import { mkdtempSync, rmSync, writeFileSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';
import { spawnSync } from 'child_process';

const hasFiveCli = (() => {
  const check = spawnSync('5ive', ['--version'], { encoding: 'utf8' });
  return check.status === 0;
})();

const maybeIt = hasFiveCli ? it : it.skip;

describe('CLI CPI compile smoke', () => {
  maybeIt('compiles a valid CPI interface script without E0004', () => {
    const tmpRoot = mkdtempSync(join(tmpdir(), 'five-cpi-smoke-'));
    const sourcePath = join(tmpRoot, 'cpi-smoke.v');
    const outputPath = join(tmpRoot, 'cpi-smoke.five');

    const source = `
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    transfer @discriminator(3) (
        source: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );
}

pub cpi_only(
    user_token_a: account @mut,
    pool_token_a_vault: account @mut,
    user_authority: account @signer,
    amount_a: u64
) {
    SPLToken.transfer(user_token_a, pool_token_a_vault, user_authority, amount_a);
}
`;

    writeFileSync(sourcePath, source, 'utf8');

    try {
      const result = spawnSync('5ive', ['compile', sourcePath, '-o', outputPath], {
        encoding: 'utf8',
      });

      expect(result.status).toBe(0);
      expect(`${result.stdout}\n${result.stderr}`).not.toContain('InvalidInstructionPointer');
      expect(`${result.stdout}\n${result.stderr}`).not.toContain('error[E0004]');
    } finally {
      rmSync(tmpRoot, { recursive: true, force: true });
    }
  });
});
