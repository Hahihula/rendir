# Release Procedure

Step-by-step guide for cutting a new release of `rendir`. This procedure is
designed to be run by a human; the CI handles the publishing and mirroring.

The procedure assumes:

- The release branch is `master`.
- The publishing crate is `rendir` (the CLI in `rendir/`).
- The publishing target is crates.io.
- The mirror target is `github.com/hahihula/rendir`.
- The Docker registry is Docker Hub (`hahihula/rendir` and
  `hahihula/rendir-alpine`).

---

## 0. CI Secrets (one-time setup)

The following variables must exist as **Protected** and **Masked** CI/CD
variables in the GitLab project (Settings > CI/CD > Variables). They are only
read on protected branches and tags, and are hidden from job logs.

| Variable          | Source                                                                                |
| ----------------- | ------------------------------------------------------------------------------------- |
| `CRATES_IO_TOKEN` | https://crates.io/settings/tokens — token with `publish` scope                        |
| `GITHUB_TOKEN`    | https://github.com/settings/tokens — fine-grained token for the `hahihula/rendir` repo, with `Contents: Read & Write` (for the mirror push) |
| `DOCKERHUB_TOKEN` | https://hub.docker.com/settings/security — Docker Hub access token with `Read & Write` |

`DOCKERHUB_USERNAME` is set as a non-secret **project variable** (default
`hahihula`) in `.gitlab-ci.yml`. Override it in the GitLab UI if you ever
migrate the Docker Hub account.

Everything else the CI references (`CI_REGISTRY_USER`, `CI_REGISTRY_PASSWORD`,
`CI_COMMIT_TAG`, `CI_PROJECT_URL`, etc.) is provided automatically by GitLab
and is unused by the Docker Hub workflow.

---

## 1. Pre-release checks

Run these locally before starting:

```bash
git checkout master
git pull origin master
git status          # working tree must be clean
cargo test          # all tests must pass
cargo build --release --package rendir
```

Verify on GitLab that the `master` branch is green (the `test` and `build`
stages both pass on the most recent commit).

---

## 2. Decide the version

This project is pre-1.0, so semver rules are looser. Use a MAJOR bump only
for breaking changes that affect users of the CLI or the `rendir-core` API
(major version is reserved for the eventual 1.0).

| Bump   | When                                                |
| ------ | --------------------------------------------------- |
| MAJOR  | Breaking change to CLI arguments or core API        |
| MINOR  | New feature, non-breaking change, new template      |
| PATCH  | Bug fix, doc fix, internal refactor with no API change |

---

## 3. Bump the version

The only crate that is published to crates.io is `rendir` (the CLI). Update
its version in `rendir/Cargo.toml`:

```toml
[package]
name = "rendir"
version = "0.2.0"   # <-- bump this
```

If `rendir-core` or `rendir-wasm` changed in a way that affects their
public API, bump those too. The CLI build will pick up the local path
dependency automatically, but if the lib is ever published independently, the
versions need to be coherent.

Commit the bump:

```bash
git add rendir/Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"
```

---

## 4. Push and tag

```bash
git push origin master

# Wait for the test+build pipeline on master to pass before tagging.

git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

Use an **annotated** tag (`-a`). The tag must be `v*` (e.g. `v0.2.0`) so the
release tooling in CI recognises it; the pipeline itself only filters on
`tags` but the release notes/title follow the `vX.Y.Z` convention.

---

## 5. Wait for the auto-pipeline

Pushing the tag triggers the `master + tags` path of the pipeline. The
following jobs run **automatically**:

| Job                     | Stage            | What it does                                          |
| ----------------------- | ---------------- | ----------------------------------------------------- |
| `test:linux:cli`        | test             | `cargo test` on Linux                                 |
| `build:linux:cli`       | build            | Builds release binary, uploads `rendir-linux-x86_64` artifact |
| `mirror:github`         | mirror           | Mirrors the tag/branch to `github.com/hahihula/rendir` |
| `github:release:create` | github-release   | Creates (or recreates) the GitHub release             |
| `release:gitlab`        | release          | Creates the GitLab release with the Linux CLI asset link |

Watch the pipeline on the GitLab CI/CD > Pipelines page. The release is
incomplete until at least the `release:gitlab`, `mirror:github`, and
`github:release:create` jobs succeed.

If `test:linux:cli` or `build:linux:cli` fails on the tag push, **stop**:
delete the tag, fix the issue, re-tag. Don't try to "patch" a broken release.

```bash
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0
# fix the issue, recommit, re-tag
```

---

## 6. Manual CI actions

The following jobs are **manual** and must be triggered from the pipeline UI
(click the "play" button next to the job on the Pipelines page). Run them in
this order:

1. **`release:crates-io`** — publishes the `rendir` crate to crates.io.
   This depends on the GitLab release asset having been built, but does not
   wait for it. Wait for `build:linux:cli` to be green before clicking.
2. **`docker:build:using-template`** — builds and pushes the canonical image
   to Docker Hub as `hahihula/rendir` (tags: `$CI_COMMIT_TAG`, `latest`).
   This is the image most users will pull. **Run this first.**
3. **`docker:build:alpine`** — builds and pushes the same image to Docker
   Hub as `hahihula/rendir-alpine` (tags: `$CI_COMMIT_TAG`, `latest`).
4. **`docker:build`** — builds and pushes an additional image to Docker
   Hub as `hahihula/rendir` (no `latest`/`tag` split by default; pushes
   both `$CI_COMMIT_TAG` and `latest`). This job has its own embedded script
   (does not use the template); kept for backward compatibility.

> **Note:** `docker:build:alpine` and `docker:build:using-template` produce
> near-identical images — both extend the same hidden `.docker:build:alpine`
> template and differ only in the image name. They are kept as two distinct
> Docker Hub repos so users can choose the `-alpine` tag explicitly. If
> future consolidation is wanted, delete `docker:build:alpine` and rename
> `docker:build:using-template` to `docker:build`.

The docker jobs only log in to Docker Hub if `DOCKERHUB_TOKEN` is set in
the project CI/CD variables.

---

## 7. Verify the release

After all manual jobs have run, sanity-check each target:

```bash
# crates.io
cargo search rendir
# → should show version 0.2.0

# install from crates.io
cargo install rendir --version 0.2.0
rendir --version

# GitHub mirror (the tag should be visible)
git ls-remote --tags https://github.com/hahihula/rendir.git | grep v0.2.0

# GitHub release page
# https://github.com/hahihula/rendir/releases/tag/v0.2.0

# GitLab release page
# https://gitlab.com/hahihula/rendir/-/releases/v0.2.0

# Docker images
docker pull hahihula/rendir:0.2.0
docker pull hahihula/rendir-alpine:0.2.0
```

---

## 8. Post-release

1. Announce on whatever channels the project uses (README badge update, blog
   post, etc.).
2. If a `CHANGELOG.md` is added in the future, populate it as part of the
   version-bump commit (this project does not currently maintain one).
3. Move on to the next cycle: create a feature branch, open a milestone, etc.

---

## Quick reference — happy-path commands

```bash
# Pre-release
git checkout master && git pull && cargo test

# Bump
$EDITOR rendir/Cargo.toml              # bump version
git add -A && git commit -m "chore: bump version to X.Y.Z"
git push origin master

# Tag (after pipeline on master is green)
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z

# Then on the GitLab pipeline page:
#   1. play release:crates-io
#   2. play docker:build:using-template  (canonical: hahihula/rendir)
#   3. play docker:build                 (also hahihula/rendir)
#   4. play docker:build:alpine          (hahihula/rendir-alpine)
```
