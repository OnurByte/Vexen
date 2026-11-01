#!/usr/bin/env node
"use strict";

// Bootstrap only: fetch the release binary for this platform, verify its
// SHA-256, cache it, then run it with the caller's arguments. Everything
// diagnostic goes to stderr because stdout belongs to the MCP protocol.

const crypto = require("node:crypto");
const fs = require("node:fs");
const https = require("node:https");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const version = require("../package.json").version;
const repo = "OnurByte/Vexen";
const MAX_ARCHIVE_BYTES = 200 * 1024 * 1024;
const TIMEOUT_MS = 60_000;

const targets = {
  "win32-x64": { triple: "x86_64-pc-windows-msvc", ext: "zip", exe: "vexen.exe" },
  "linux-x64": { triple: "x86_64-unknown-linux-gnu", ext: "tar.gz", exe: "vexen" },
  "linux-arm64": { triple: "aarch64-unknown-linux-gnu", ext: "tar.gz", exe: "vexen" },
  "darwin-x64": { triple: "x86_64-apple-darwin", ext: "tar.gz", exe: "vexen" },
  "darwin-arm64": { triple: "aarch64-apple-darwin", ext: "tar.gz", exe: "vexen" },
};

function log(message) {
  process.stderr.write(`vexen: ${message}\n`);
}

function fail(message) {
  log(message);
  process.exit(1);
}

function cacheRoot() {
  if (process.env.VEXEN_HOME) return process.env.VEXEN_HOME;
  if (process.platform === "win32" && process.env.LOCALAPPDATA) {
    return path.join(process.env.LOCALAPPDATA, "vexen");
  }
  return path.join(process.env.XDG_CACHE_HOME || path.join(os.homedir(), ".cache"), "vexen");
}

function get(url, redirects = 5) {
  return new Promise((resolve, reject) => {
    const request = https.get(url, { headers: { "user-agent": "vexen-npm-bootstrap" } }, (res) => {
      if ([301, 302, 303, 307, 308].includes(res.statusCode) && res.headers.location) {
        res.resume();
        if (redirects === 0) return reject(new Error("too many redirects"));
        const next = new URL(res.headers.location, url);
        if (next.protocol !== "https:") return reject(new Error("refusing a non-HTTPS redirect"));
        return resolve(get(next.toString(), redirects - 1));
      }
      if (res.statusCode !== 200) {
        res.resume();
        return reject(Object.assign(new Error(`HTTP ${res.statusCode} for ${url}`), { status: res.statusCode }));
      }
      const chunks = [];
      let size = 0;
      res.on("data", (chunk) => {
        size += chunk.length;
        if (size > MAX_ARCHIVE_BYTES) return request.destroy(new Error("download exceeds the size limit"));
        chunks.push(chunk);
      });
      res.on("end", () => resolve(Buffer.concat(chunks)));
      res.on("error", reject);
    });
    request.setTimeout?.(TIMEOUT_MS, () => request.destroy(new Error("download timed out")));
    request.on("error", reject);
  });
}

function tarCommand() {
  // Only bsdtar reads zip; Git Bash can put GNU tar first on PATH.
  if (process.platform === "win32") {
    return path.join(process.env.SystemRoot || "C:\\Windows", "System32", "tar.exe");
  }
  return "tar";
}

async function install(target, binary) {
  const name = `vexen-${version}-${target.triple}`;
  const base = `https://github.com/${repo}/releases/download/v${version}`;
  let archive;
  let checksum;
  try {
    [archive, checksum] = await Promise.all([
      get(`${base}/${name}.${target.ext}`),
      get(`${base}/${name}.${target.ext}.sha256`),
    ]);
  } catch (error) {
    if (error.status === 404) {
      fail(
        `no release binary is published for v${version} on ${target.triple} yet. ` +
          `Build from source instead: cargo install --git https://github.com/${repo} --locked`,
      );
    }
    fail(`download failed: ${error.message}`);
  }
  const expected = checksum.toString("utf8").trim().split(/\s+/)[0].toLowerCase();
  const actual = crypto.createHash("sha256").update(archive).digest("hex");
  if (!/^[0-9a-f]{64}$/.test(expected) || expected !== actual) {
    fail("checksum mismatch; refusing to run the downloaded archive.");
  }

  // Unpack in a private temp directory, then rename: two parallel runs never
  // see (or run) a half-written binary.
  const dir = path.dirname(binary);
  fs.mkdirSync(path.dirname(dir), { recursive: true });
  const work = fs.mkdtempSync(`${dir}.tmp-`);
  try {
    const archivePath = path.join(work, `${name}.${target.ext}`);
    fs.writeFileSync(archivePath, archive);
    const unpacked = spawnSync(tarCommand(), ["-xf", archivePath, "-C", work], {
      stdio: ["ignore", process.stderr, process.stderr],
    });
    const staged = path.join(work, target.exe);
    if (unpacked.status !== 0 || !fs.existsSync(staged)) {
      fail("could not unpack the downloaded archive (is `tar` available?).");
    }
    if (process.platform !== "win32") fs.chmodSync(staged, 0o755);
    fs.mkdirSync(dir, { recursive: true });
    try {
      fs.renameSync(staged, binary);
    } catch (error) {
      if (!fs.existsSync(binary)) throw error; // lost a race to an identical copy
    }
  } finally {
    fs.rmSync(work, { recursive: true, force: true });
  }
}

async function main() {
  let binary = process.env.VEXEN_BINARY;
  if (!binary) {
    const target = targets[`${process.platform}-${process.arch}`];
    if (!target) {
      fail(
        `unsupported platform ${process.platform}-${process.arch}. ` +
          `Build from source: cargo install --git https://github.com/${repo} --locked`,
      );
    }
    binary = path.join(cacheRoot(), version, target.triple, target.exe);
    if (!fs.existsSync(binary)) {
      log(`downloading v${version} for ${target.triple}`);
      await install(target, binary);
    }
  }
  const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
  if (result.error) fail(`could not start ${binary}: ${result.error.message}`);
  if (result.signal) process.kill(process.pid, result.signal);
  process.exit(result.status === null ? 1 : result.status);
}

main().catch((error) => fail(error.message));
