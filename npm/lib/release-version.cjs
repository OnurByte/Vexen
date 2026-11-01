"use strict";

// Pure release policy: patch releases are automatic, minor/major releases
// require an explicit trailer on the tagged commit. Never infer a bump from
// a package version string or silently renumber published tags.
const TAG = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;

function parseTag(tag) {
  const match = typeof tag === "string" ? TAG.exec(tag) : null;
  if (!match) throw new Error("Expected a complete vMAJOR.MINOR.PATCH tag (for example v0.2.3).");
  const parts = match.slice(1).map(Number);
  if (!parts.every(Number.isSafeInteger)) throw new Error("Release version components must be safe integers.");
  return { major: parts[0], minor: parts[1], patch: parts[2] };
}

function compareVersions(a, b) {
  return a.major - b.major || a.minor - b.minor || a.patch - b.patch;
}

function validateRelease({ tag, previousTag, cargoVersion, npmVersion, releaseBump = "" }) {
  const current = parseTag(tag);
  const previous = parseTag(previousTag);
  const version = tag.slice(1);

  if (cargoVersion !== version || npmVersion !== version) {
    throw new Error("Cargo.toml and npm/package.json must both match the release tag.");
  }

  const expected = `v${previous.major}.${previous.minor}.${previous.patch + 1}`;
  const unexpected = () => {
    throw new Error(
      `Unexpected release jump from ${previousTag} to ${tag}; expected next patch ${expected}. ` +
      "Intentional minor/major releases must advance exactly one component and carry " +
      "Release-Bump: minor or Release-Bump: major on the tagged commit."
    );
  };

  if (current.major === previous.major && current.minor === previous.minor
      && current.patch === previous.patch + 1) {
    return { version, bump: "patch" };
  }

  if (current.major === previous.major && current.minor === previous.minor + 1
      && current.patch === 0) {
    if (releaseBump !== "minor") throw new Error("An intentional minor release requires Release-Bump: minor.");
    return { version, bump: "minor" };
  }

  if (current.major === previous.major + 1 && current.minor === 0
      && current.patch === 0) {
    if (releaseBump !== "major") throw new Error("An intentional major release requires Release-Bump: major.");
    return { version, bump: "major" };
  }

  return unexpected();
}

module.exports = { parseTag, compareVersions, validateRelease };
