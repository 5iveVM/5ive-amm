#!/usr/bin/env node

import { loadDslFeatureMatrix, repoRootFrom } from './lib/dsl-feature-matrix.mjs';

const repoRoot = repoRootFrom(import.meta.url);
loadDslFeatureMatrix(repoRoot);
console.log('DSL feature matrix is valid.');
