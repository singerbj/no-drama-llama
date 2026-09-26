# Releasing

## One-time setup: the update signing key

The in-app updater only installs releases signed with your minisign key. Without a key,
releases still publish, but the app can only tell users that a new version exists.

1. Install [minisign](https://jedisct1.github.io/minisign/) and create a key pair without a
   password (GitHub's secret store protects it):

   ```sh
   minisign -G -W -p minisign.pub -s minisign.key
   ```

2. In the repository settings, go to **Secrets and variables → Actions**:
   - **Variables** → `UPDATE_PUBKEY`: the second line of `minisign.pub` (the base64 key).
   - **Secrets** → `UPDATE_SIGNING_KEY`: the full contents of `minisign.key`.
3. Store `minisign.key` offline somewhere safe, then delete the local copy.

The public key is compiled into every release build. If you rotate it, the next release must
still be signed with the old key, because installed apps only trust the key they were built
with.

## Cutting a release

1. Bump `version` in `Cargo.toml` and run `cargo check` to update `Cargo.lock`.
2. Add a `CHANGELOG.md` entry.
3. Commit, then tag and push:

   ```sh
   git tag v2.0.1 && git push origin v2.0.1
   ```

The **Release** workflow then:
1. checks that the tag matches the crate version
2. runs the tests
3. builds the exe with the public key baked in
4. signs it with trusted comment `no-drama-llama <version>`, verifies the signature, and writes
   `SHA256SUMS`
5. publishes the release

Installed apps with *Update automatically* on pick it up within 24 hours. Users can also choose
*Check for updates* in the tray menu or run `no-drama-llama.exe update`.

## Code signing (recommended next step)

The exe isn't Authenticode-signed yet, so SmartScreen warns on first download. Add a signing
step for `no-drama-llama.exe` to the `build` job, for example with Azure Trusted Signing. Sign
before the minisign step, so the minisign signature covers the Authenticode-signed file.
