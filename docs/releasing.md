# Releasing

No Drama Llama, rekt clipz and TunedUp release, sign and update the same way: the same
workflows, the same [`scripts/release/`](../scripts/release/) (only `config.ts` differs), the
same settings names and the same updater rules. Change one, change all three.

A release is a GitHub release tagged `vX.Y.Z` (the `version` in `Cargo.toml`) holding:

| File | What it is |
| --- | --- |
| `no-drama-llama.exe` | The app. It installs itself when run. Authenticode-signed once code signing is set up. |
| `no-drama-llama.exe.minisig` | Its minisign signature. The trusted comment names the version (Tauri's format, `timestamp:…\tfile:…\tversion:X.Y.Z`). |
| `latest.json` | The updater manifest in Tauri's format: version, notes, and the exe's URL and signature. Installed apps read it. |
| `SHA256SUMS` | Checksums of the three files above. |

## Cutting a release

**Actions → Prepare release → Run workflow** on `main`. Pick the bump (`patch`, `minor`,
`major`, or a `pre*` bump for a beta) or type an exact version, and tick *Dry run* to see the
diff without pushing. After you approve the `release-prep` environment it:

1. bumps the version in `Cargo.toml` and `Cargo.lock` and moves the changelog's
   `[Unreleased]` notes under it ([`scripts/release/version.ts`](../scripts/release/version.ts)).
   It stops if `[Unreleased]` is empty.
2. commits `chore(release): vX.Y.Z` to `main` and pushes the tag `vX.Y.Z` in one atomic push,
   so the tag only exists if `main` took the commit.
3. starts **Release** on the tag.

**Release** then runs four jobs:

1. **Version**: the tag must match `Cargo.toml` and point at a commit on `main`.
2. **Build** (Windows): runs the tests, builds the exe with the updater's public key baked in,
   and smoke-tests it.
3. **Code signing**: Authenticode through SignPath or a certificate, if set up (below). It checks
   the signed file is the CI build byte for byte plus a valid, timestamped signature. Without
   code signing the exe passes through unsigned.
4. **Publish** (waits for approval on the `release` environment): signs the exe for the updater
   and checks the signature against `UPDATE_PUBKEY`, writes `latest.json` (notes generated from
   the PRs since the previous release) and `SHA256SUMS`, uploads everything to a draft release,
   downloads it again to check the checksums, and publishes it. Versions with a pre-release
   part (`1.2.0-beta.1`) are published as GitHub pre-releases and never become *latest*.

By hand instead: `node scripts/release/version.ts bump patch` (or `set 1.2.3`), commit, and
push the tag `node scripts/release/version.ts tag` prints. To retry a release that failed, run
**Release** on its tag (*Use workflow from → Tags*). A release that's already published is
never changed: release a new version instead.

## How installed apps update

With *Update automatically* on (the default), the app reads
`https://github.com/singerbj/no-drama-llama/releases/latest/download/latest.json` a minute
after it starts and every 6 hours (30 minutes after a failed check). *Check for updates* in the
tray menu, or `no-drama-llama.exe update`, checks now. It installs a release only if:

1. it's newer than the running version and not a pre-release;
2. the exe's minisign signature checks out against the public key compiled into the app
   (`NDL_UPDATE_PUBKEY`, from the `UPDATE_PUBKEY` variable);
3. the signature names that exact version, so a validly signed old build can't be passed off as
   a new one.

It then swaps the exe in the install folder and restarts into it; `llama-server` keeps running.
Builds without the key (local builds, forks) only say that a new version exists.

## One-time setup

### Updater signing key

Run this on your own machine, with the [GitHub CLI](https://cli.github.com) logged in as a
repository admin:

```sh
node scripts/release/setup-secrets.ts
```

It creates a password-protected key pair in `~/.release-keys/` (with `tauri signer generate`,
or pass `--key <file>` to use one you have), and stores:

| Where | Name | Value |
| --- | --- | --- |
| Repository variable | `UPDATE_PUBKEY` | the public key (any minisign format) |
| `release` environment secret | `UPDATE_SIGNING_KEY` | the secret key (minisign or `tauri signer` format) |
| `release` environment secret | `UPDATE_SIGNING_KEY_PASSWORD` | its password |

It also creates the `release-prep`, `codesign` and `release` environments, limited to `main`
or to `v*` tags, with you as a required reviewer of `release-prep` and `release`.

Back up the key file and its password. The public key is compiled into every build, so if the
key is lost, installed copies can never update themselves again, and a new key only reaches
users who install a new build by hand.

Then, under **Settings → Rules → Rulesets**, add a tag ruleset for `v*` that restricts
creation, update and deletion to administrators (and the release App below).

### Release App (optional)

Without it, Prepare release pushes with `GITHUB_TOKEN`, which needs Actions to be allowed to
push to `main` and can't bypass rulesets. With a GitHub App it can:

1. Create a GitHub App (**Settings → Developer settings → GitHub Apps**) with no webhook and only
   the repository permission **Contents: Read and write**, and install it on this repository.
2. Add the repository variable `RELEASE_APP_CLIENT_ID` (its client ID) and the `release-prep`
   environment secret `RELEASE_APP_PRIVATE_KEY` (a private key it generates).
3. Add the App to the bypass list of the `v*` tag ruleset and of whatever protects `main`.

### Code signing (optional)

Without an Authenticode signature, SmartScreen shows "Windows protected your PC" on first run.
The **Code signing** job uses one of these, on the `codesign` environment:

- **SignPath** ([SignPath Foundation](https://signpath.org/) signs open-source projects for
  free): the variable `SIGNPATH_ORGANIZATION_ID` and the secret `SIGNPATH_API_TOKEN`
  (optionally `SIGNPATH_PROJECT_SLUG`, default `no-drama-llama`, and
  `SIGNPATH_SIGNING_POLICY_SLUG`, default `release-signing`). SignPath fetches the exe from the
  run and signs it once a project approver approves the request on signpath.io; the job waits
  up to two hours. Set it up like this:
  1. [Apply to SignPath Foundation](https://signpath.org/apply). The README's
     [Code signing policy](https://github.com/singerbj/no-drama-llama#code-signing-policy)
     section covers what they ask the project to publish. Every GitHub account with write
     access needs two-factor authentication.
  2. On [signpath.io](https://app.signpath.io), install the
     [SignPath GitHub App](https://github.com/apps/signpath) on this repository, link the
     predefined **GitHub.com** trusted build system to the project, paste
     [`.signpath/artifact-configuration.xml`](../.signpath/artifact-configuration.xml) into the
     project's artifact configuration (it only signs `no-drama-llama.exe` with product name
     *No Drama Llama* and the release's version), and create an API token for a CI user who
     may submit to the signing policy.
- **A certificate**: the secrets `WINDOWS_CERTIFICATE` (the base64 of the `.pfx`) and
  `WINDOWS_CERTIFICATE_PASSWORD`, signed with `signtool` and timestamped.

`CODESIGN_PUBLISHER` (a variable) is the signer name the job insists on; it defaults to
*SignPath Foundation* with SignPath. Setting half of the SignPath pair, or both SignPath and a
certificate, fails the release instead of quietly shipping an unsigned exe.

SmartScreen can still warn about the first few signed releases while the certificate builds
reputation. After the first signed release, update the "Windows protected your PC" answers in
the README, the troubleshooting and install pages, and the website FAQ.

## Rolling back

Installed apps never downgrade. To stop a bad release from spreading, mark it a pre-release
(or delete it) on GitHub so the previous release is *latest* again, then release a fixed,
higher version.
