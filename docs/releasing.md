# Releasing

## One-time setup: the update signing key

The in-app updater only installs releases signed with your minisign key. Without a key,
releases still publish, but the app can only tell users that a new version exists.

1. Install [minisign](https://jedisct1.github.io/minisign/) and create a key pair without a
   password (GitHub's secret store protects it):

   ```sh
   minisign -G -W -p minisign.pub -s minisign.key
   ```

2. In the repository settings, go to **Secrets and variables → Actions** → **Variables** and
   add `UPDATE_PUBKEY`: the second line of `minisign.pub` (the base64 key).
3. Go to **Environments**, create an environment named `release`, and configure it:
   - **Deployment branches and tags** → *Selected branches and tags* → add the tag rule `v*`.
   - **Required reviewers** → yourself (so every signed release needs a click to approve).
   - **Environment secrets** → `UPDATE_SIGNING_KEY`: the full contents of `minisign.key`.

   Keep the key off the repository-level secrets: those are readable by a workflow on any
   pushed branch, while an environment secret only reaches the approved `publish` job.
4. Under **Rules → Rulesets**, add a tag ruleset for `v*` that restricts creation, update and
   deletion to administrators.
5. Store `minisign.key` offline somewhere safe, then delete the local copy.

The public key is compiled into every release build. If you rotate it, the next release must
still be signed with the old key, because installed apps only trust the key they were built
with.

## Cutting a release

### From the Actions tab

Once the one-time setup below is done, go to **Actions → Prepare release → Run workflow** on
`main` and pick the bump (patch, minor or major) or type an exact version. After you approve the
`release-prep` environment, it:
1. bumps `version` in `Cargo.toml` and `Cargo.lock`
   ([`scripts/prepare-release.ts`](../scripts/prepare-release.ts)) and moves the changelog's
   `[Unreleased]` notes under the new version. It stops if `[Unreleased]` is empty.
2. commits `Release vX.Y.Z` to `main` and pushes the tag `vX.Y.Z` in one atomic push, so the
   tag is only created if `main` took the commit. If someone merged to `main` in the meantime,
   the push fails and you can run it again.
3. the tag starts the **Release** workflow below, with its usual approvals.

Tick *Dry run* to see the version and diff without pushing anything.

The same script works locally: `node scripts/prepare-release.ts minor`.

#### One-time setup: the release App

The workflow pushes with a GitHub App's token instead of `GITHUB_TOKEN`: only administrators
may create `v*` tags, and a tag pushed with `GITHUB_TOKEN` wouldn't start the Release workflow.

1. Create a GitHub App under **Settings → Developer settings → GitHub Apps → New GitHub App**
   (for example *no-drama-llama-release*): no webhook, the repository permission
   **Contents: Read and write** and nothing else, installable only on your account. Install it
   on this repository only.
2. Note its **Client ID** and generate a **private key**.
3. In the repository settings:
   - **Secrets and variables → Actions → Variables**: add `RELEASE_APP_CLIENT_ID`.
   - **Environments**: create `release-prep`, then set **Deployment branches and tags** to the
     branch `main`, **Required reviewers** to yourself, and the **environment secret**
     `RELEASE_APP_PRIVATE_KEY` to the contents of the `.pem` file.
   - **Rules → Rulesets**: add the App to the bypass list of the `v*` tag ruleset and of
     whatever protects `main` (required pull requests, status checks), so it can push the
     release commit and the tag.
4. Delete the local `.pem` file.

### By hand

1. Bump `version` in `Cargo.toml` and run `cargo check` to update `Cargo.lock`.
2. Add a `CHANGELOG.md` entry.
3. Commit, then tag and push:

   ```sh
   git tag v2.0.1 && git push origin v2.0.1
   ```

The **Release** workflow then:
1. checks that the tagged commit is on `main` and that the tag matches the crate version
2. runs the tests
3. builds the exe with the public key baked in
4. has SignPath Authenticode-sign it, if that's set up (see below)
5. waits for approval on the `release` environment, then signs it with trusted comment `no-drama-llama <version>`, verifies the signature, and writes
   `SHA256SUMS`
6. publishes the release

Installed apps with *Update automatically* on pick it up within 24 hours. Users can also choose
*Check for updates* in the tray menu or run `no-drama-llama.exe update`.

## Optional setup: Windows code signing (SignPath Foundation)

Without an Authenticode signature, SmartScreen shows "Windows protected your PC" on first run.
[SignPath Foundation](https://signpath.org/) signs open-source projects for free. The release
workflow's `codesign` job uses it once it's set up, and passes the exe through unsigned until
then.

1. [Apply to SignPath Foundation](https://signpath.org/apply). The README's
   [Code signing policy](https://github.com/singerbj/no-drama-llama#code-signing-policy)
   section covers what they ask the project to publish. Every GitHub account with write access
   needs two-factor authentication.
2. Once you're accepted, set up the project on [signpath.io](https://app.signpath.io):
   - Install the [SignPath GitHub App](https://github.com/apps/signpath) on this repository
     and link the predefined **GitHub.com** trusted build system to the project.
   - Name the project `no-drama-llama`, or set the repository variable `SIGNPATH_PROJECT_SLUG`.
   - Paste [`.signpath/artifact-configuration.xml`](../.signpath/artifact-configuration.xml)
     into the project's artifact configuration and make it the default. It only signs
     `no-drama-llama.exe` with product name *No Drama Llama* and the release's version.
   - Use the `release-signing` signing policy (or set `SIGNPATH_SIGNING_POLICY_SLUG`), and
     create an API token for a CI user who may submit to it.
3. In the repository settings, add the variable `SIGNPATH_ORGANIZATION_ID` (from signpath.io),
   then go to **Environments**, create `codesign`, and configure it:
   - **Deployment branches and tags** → *Selected branches and tags* → add the tag rule `v*`.
   - **Environment secrets** → `SIGNPATH_API_TOKEN`: the token from step 2.

   It doesn't need required reviewers: SignPath asks a project approver to approve every
   signing request, and the workflow waits up to two hours for that.

With both set, each release goes like this:
1. SignPath fetches the exe straight from the workflow run and signs it once you approve the
   request on signpath.io.
2. The `codesign` job checks the result:
   - [`scripts/check-signed-exe.ts`](../scripts/check-signed-exe.ts) confirms it's the CI
     build byte for byte, plus a signature.
   - `Get-AuthenticodeSignature` confirms the signature is valid, timestamped, and from
     SignPath Foundation.
3. The `publish` job signs that file with minisign, so the updater's signature covers the
   Authenticode-signed exe.

If only one of `SIGNPATH_ORGANIZATION_ID` and `SIGNPATH_API_TOKEN` is set, the release fails
instead of quietly shipping an unsigned exe.

SmartScreen can still warn about the first few signed releases while the certificate builds
reputation. After the first signed release, update the "Windows protected your PC" answers in
the README, the troubleshooting and install pages, and the website FAQ.
