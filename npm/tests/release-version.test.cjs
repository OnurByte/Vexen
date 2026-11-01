"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const { validateRelease } = require("../lib/release-version.cjs");

const base = {
  tag: "v0.2.3",
  previousTag: "v0.2.2",
  cargoVersion: "0.2.3",
  npmVersion: "0.2.3",
  releaseBump: "",
};

test("next release is a normal patch, not v2", () => {
  assert.deepEqual(validateRelease(base), { version: "0.2.3", bump: "patch" });
});

test("rejects incomplete tags like v2", () => {
  assert.throws(() => validateRelease({ ...base, tag: "v2" }), /vMAJOR\.MINOR\.PATCH/);
});

test("rejects a skipped patch version", () => {
  assert.throws(() => validateRelease({
    ...base,
    tag: "v0.2.4",
    cargoVersion: "0.2.4",
    npmVersion: "0.2.4",
  }), /next patch/);
});

test("rejects an unapproved minor release", () => {
  assert.throws(() => validateRelease({
    ...base,
    tag: "v0.3.0",
    cargoVersion: "0.3.0",
    npmVersion: "0.3.0",
  }), /Release-Bump: minor/);
});

test("allows an intentional minor release", () => {
  assert.deepEqual(validateRelease({
    ...base,
    tag: "v0.3.0",
    cargoVersion: "0.3.0",
    npmVersion: "0.3.0",
    releaseBump: "minor",
  }), { version: "0.3.0", bump: "minor" });
});

test("rejects a major jump even with a major marker", () => {
  assert.throws(() => validateRelease({
    ...base,
    tag: "v2.0.0",
    cargoVersion: "2.0.0",
    npmVersion: "2.0.0",
    releaseBump: "major",
  }), /next patch/);
});

test("allows a deliberate next major release", () => {
  assert.deepEqual(validateRelease({
    ...base,
    tag: "v1.0.0",
    cargoVersion: "1.0.0",
    npmVersion: "1.0.0",
    releaseBump: "major",
  }), { version: "1.0.0", bump: "major" });
});

test("rejects cargo/npm version drift", () => {
  assert.throws(() => validateRelease({ ...base, npmVersion: "0.2.2" }), /Cargo\.toml and npm\/package\.json/);
});

test("rejects downgrades and arbitrary minor patches", () => {
  assert.throws(() => validateRelease({
    ...base,
    tag: "v0.1.9",
    cargoVersion: "0.1.9",
    npmVersion: "0.1.9",
  }), /next patch/);
  assert.throws(() => validateRelease({
    ...base,
    tag: "v0.3.1",
    cargoVersion: "0.3.1",
    npmVersion: "0.3.1",
    releaseBump: "minor",
  }), /next patch/);
});
