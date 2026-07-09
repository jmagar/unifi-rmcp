"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const {
  downloadUrl,
  releaseBaseUrl,
  releaseVersion,
  targetFor,
} = require("../lib/platform");
const { version: packageVersion } = require("../package.json");

test("maps supported platforms to release assets", () => {
  assert.deepEqual(targetFor("linux", "x64"), {
    asset: "runifi-x86_64.tar.gz",
    binary: "runifi",
  });
  assert.deepEqual(targetFor("win32", "x64"), {
    asset: "runifi-windows-x86_64.tar.gz",
    binary: "runifi.exe",
  });
});

test("rejects unsupported platforms", () => {
  assert.throws(() => targetFor("darwin", "arm64"), /Unsupported platform/);
});

test("uses npm package version as the binary tag by default", () => {
  assert.equal(releaseVersion({}), `v${packageVersion}`);
});

test("allows release tag and repo overrides", () => {
  const env = {
    UNIFI_RMCP_BINARY_VERSION: "v9.9.9",
    UNIFI_RMCP_REPO: "example/unifi-rmcp",
  };
  assert.equal(releaseBaseUrl(env), "https://github.com/example/unifi-rmcp/releases/download");
  assert.equal(
    downloadUrl(targetFor("linux", "x64"), env),
    "https://github.com/example/unifi-rmcp/releases/download/v9.9.9/runifi-x86_64.tar.gz",
  );
});
