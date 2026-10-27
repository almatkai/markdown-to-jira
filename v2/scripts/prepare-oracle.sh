#!/bin/sh
# Run from any location; all disposable files stay in v2/.work.
set -eu
cd "$(dirname "$0")/.."
mkdir -p .work/own
cp -R tests/fixtures/own/. .work/own/
pnpm import --dir .work/own
pnpm install --dir .work/own --frozen-lockfile --ignore-scripts
pnpm --dir .work/own test
pnpm --dir .work/own typecheck
node scripts/generate-oracle.cjs
