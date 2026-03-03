import { mkdir, readFile, writeFile } from 'fs/promises';
import { dirname } from 'path';

import { TypeGenerator, normalizeAbiFunctions } from '@5ive-tech/sdk';

export function normalizeArtifactAbi(abiData: any): any | undefined {
  if (!abiData) {
    return undefined;
  }

  const base =
    abiData && typeof abiData === 'object' && !Array.isArray(abiData)
      ? { ...abiData }
      : { version: '1.0' };

  return {
    ...base,
    functions: normalizeAbiFunctions((abiData as any).functions ?? abiData),
    version: base.version || '1.0',
  };
}

export function createFiveArtifact(params: {
  bytecode: Uint8Array;
  abi?: any;
  metadata?: any;
  version?: string;
}): any {
  const normalizedAbi = normalizeArtifactAbi(params.abi);

  return {
    bytecode: Buffer.from(params.bytecode).toString('base64'),
    abi: normalizedAbi,
    version: params.version || '1.0',
    metadata: params.metadata || {},
  };
}

export async function writePackagedArtifact(
  outputFile: string,
  artifact: any,
  options: { emitTypes?: boolean } = {},
): Promise<{ artifactBuffer: Buffer; typeFile?: string }> {
  await mkdir(dirname(outputFile), { recursive: true });
  const serialized = JSON.stringify(artifact, null, 2);
  await writeFile(outputFile, serialized);

  let typeFile: string | undefined;
  if (options.emitTypes && artifact?.abi) {
    typeFile = await writeTypeDefinitions(outputFile, artifact.abi);
  }

  return {
    artifactBuffer: Buffer.from(serialized),
    typeFile,
  };
}

export async function writeTypeDefinitions(outputFile: string, abi: any): Promise<string> {
  const generator = new TypeGenerator(abi);
  const typeDefs = generator.generate();
  const typeFile = outputFile.replace(/(\.five|\.bin|\.fbin|\.so)$/, '') + '.d.ts';
  await writeFile(typeFile, typeDefs);
  return typeFile;
}

export async function writeAbiFile(outputFile: string, abi: any): Promise<void> {
  await mkdir(dirname(outputFile), { recursive: true });
  await writeFile(outputFile, JSON.stringify(abi, null, 2));
}

export async function readBytecodeFile(
  filePath: string,
  encoding: 'binary' | 'hex' = 'binary',
): Promise<Uint8Array> {
  if (encoding === 'hex') {
    const hexText = (await readFile(filePath, 'utf8')).trim().replace(/\s+/g, '');
    if (hexText.length % 2 !== 0) {
      throw new Error(`Hex bytecode file must contain an even number of characters: ${filePath}`);
    }
    return new Uint8Array(Buffer.from(hexText, 'hex'));
  }

  return new Uint8Array(await readFile(filePath));
}
