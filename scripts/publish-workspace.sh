#!/usr/bin/env bash
set -euo pipefail

version=$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)
packages=(
  zavora-docx-opc
  zavora-docx-oxml
  zavora-docx-layout
  zavora-docx-html
  zavora-docx-pdf
  zavora-docx
  zavora-docx-cli
)

for package in "${packages[@]}"; do
  if curl --fail --silent --show-error \
    --user-agent "zavora-docx-release/${version}" \
    "https://crates.io/api/v1/crates/${package}/${version}" >/dev/null 2>&1; then
    echo "${package} ${version} is already published"
    continue
  fi
  cargo publish --locked --no-verify -p "${package}"
done
