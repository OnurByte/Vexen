#!/usr/bin/env node
"use strict";

// This is run on a pushed release tag, before creating a GitHub release.
const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const { parseTag, compareVersions, validateRelease } = require("../lib/release-version.cjs");

const repoRoot = path.resolve(__dirname, "..", "..");

function git(...args) {
  return execFileSync("git", args, { cwd: repoRoot, encoding: "utf8" }).trim();
}

function main() {
  const tag = process.argv[2] || process.env.GITHUB_REF_NAME;
  const current = parseTag(tag);
  const manifest = fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
  const cargoVersion = /^version\s*=\s*"([^"]+)"/m.exec(manifest)?.[1];
  const npmVersion = require("../package.json").version;
  if (!cargoVersion) throw new Error("Cargo.toml package version not found.");

  // Compare numerical SemVer components; string sorting puts v0.10 before v0.9.
  const previousTag = git("tag", "--list").split("\n")
    .filter(Boolean)
    .flatMap((candidate) => {
      try {
        const parsed = parseTag(candidate);
        return compareVersions(parsed, current) < 0 ? [{ tag: candidate, parsed }] : [];
      } catch {
        return []; // Ignore unrelated/non-SemVer tags.
      }
    })
    .sort((a, b) => compareVersions(b.parsed, a.parsed))[0]?.tag;
  if (!previousTag) throw new Error("No prior release tag found; first release needs explicit review.");

  const message = git("log", "-1", "--format=%B", tag);
  const releaseBump = Array.from(message.matchAll(/^Release-Bump:\s*(minor|major)\s*$/gm))
    .at(-1)?.[1] || "";

  const result = validateRelease({
    tag, previousTag, cargoVersion, npmVersion, releaseBump,
  });
  process.stdout.write(
    `Release version accepted: ${tag} (${result.bump}, previous ${previousTag})\n`
  );
}

try {
  main();
} catch (error) {
  process.stderr.write(`Release rejected: ${error.message}\n`);
  process.exitCode = 1;
}
