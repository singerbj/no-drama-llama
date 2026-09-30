// This repository's release settings. Everything else in scripts/release/ is the same in every
// desktop app (no-drama-llama, rekt-clipz, tunedup): change it there too.
export const config = {
  /** Lowercase name: release asset names, the signing key's file name. */
  slug: "no-drama-llama",
  productName: "No Drama Llama",
  repo: "singerbj/no-drama-llama",
  /** Release tags are `<tagPrefix><version>`. */
  tagPrefix: "v",
  /** What users download and the updater installs: the self-installing app. */
  asset: "no-drama-llama.exe",
  /** `latest.json` platform keys the asset is listed under. */
  platforms: ["windows-x86_64"],
  /** Holds the version (`[package]` or `[workspace.package]`). */
  cargoToml: "Cargo.toml",
  cargoLock: "Cargo.lock",
  /** This repo's crates in Cargo.lock (they move with the version). */
  crates: /^no-drama-llama$/,
  packageJsons: [] as string[],
  packageLock: undefined as string | undefined,
  /** Keep-a-Changelog file whose [Unreleased] notes each release takes, if any. */
  changelog: "CHANGELOG.md" as string | undefined,
};
