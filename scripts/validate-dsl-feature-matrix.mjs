#!/usr/bin/env node

import {
  assertNoUnclassifiedCanonicalFixtures,
  loadDslBuiltinMatrix,
  loadDslFeatureInventory,
  loadDslFeatureMatrix,
  repoRootFrom,
} from './lib/dsl-feature-matrix.mjs';

const repoRoot = repoRootFrom(import.meta.url);
const featureMatrix = loadDslFeatureMatrix(repoRoot);
loadDslBuiltinMatrix(repoRoot);
const featureInventory = loadDslFeatureInventory(repoRoot);
assertNoUnclassifiedCanonicalFixtures(repoRoot, featureMatrix, featureInventory);
console.log('DSL feature matrix, builtin matrix, and feature inventory are valid.');
