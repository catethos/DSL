#!/usr/bin/env node
'use strict';

const { existsSync } = require('fs');
const { join } = require('path');
const { execSync } = require('child_process');

const root = join(__dirname, '..');

// Check if prebuilt binary exists (from optional dependencies)
function tryLoadPrebuilt() {
  const platform = process.platform;
  const arch = process.arch;

  // Map to package names
  const platformMap = {
    'darwin-arm64': 'lattice-lang-darwin-arm64',
    'darwin-x64': 'lattice-lang-darwin-x64',
    'linux-arm64': 'lattice-lang-linux-arm64-gnu',
    'linux-x64': 'lattice-lang-linux-x64-gnu',
    'win32-x64': 'lattice-lang-win32-x64-msvc',
  };

  const key = `${platform}-${arch}`;
  const packageName = platformMap[key];

  if (!packageName) {
    console.log(`No prebuilt binary available for ${key}`);
    return false;
  }

  try {
    // Try to resolve the optional dependency
    const prebuiltPath = require.resolve(`${packageName}/lattice.${platform === 'win32' ? 'dll' : platform === 'darwin' ? 'dylib' : 'so'}`);
    console.log(`Found prebuilt binary: ${prebuiltPath}`);
    return true;
  } catch (e) {
    // Optional dependency not installed
    return false;
  }
}

// Build from source
function buildFromSource() {
  console.log('Building from source...');
  console.log('This requires Rust to be installed. Get it from https://rustup.rs/');

  try {
    execSync('npm run build-release', {
      cwd: root,
      stdio: 'inherit',
    });
    console.log('Build successful!');
    return true;
  } catch (e) {
    console.error('Build failed:', e.message);
    return false;
  }
}

// Main
function main() {
  // Check if index.node already exists (local development)
  if (existsSync(join(root, 'index.node'))) {
    console.log('Native module already exists');
    return;
  }

  // Try prebuilt first
  if (tryLoadPrebuilt()) {
    return;
  }

  // Fall back to building from source
  if (!buildFromSource()) {
    console.error('\nFailed to install lattice-lang.');
    console.error('Please ensure Rust is installed: https://rustup.rs/');
    process.exit(1);
  }
}

main();
