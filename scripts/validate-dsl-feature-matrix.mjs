#!/usr/bin/env node

import {
  loadDslBuiltinMatrix,
  loadDslFeatureInventory,
  loadDslFeatureMatrix,
  repoRootFrom,
} from './lib/dsl-feature-matrix.mjs';

const repoRoot = repoRootFrom(import.meta.url);
loadDslFeatureMatrix(repoRoot);
loadDslBuiltinMatrix(repoRoot);
loadDslFeatureInventory(repoRoot);
console.log('DSL feature matrix, builtin matrix, and feature inventory are valid.');
