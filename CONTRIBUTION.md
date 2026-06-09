# Contributing to Rendir

This guide covers the two operational pieces you need to know as a
maintainer:

1. The CI secrets that must exist in **GitLab** and **GitHub** before the
   pipelines will publish anything.
2. The release procedure (mirrored from the canonical
   [`RELEASE.md`](./RELEASE.md), kept in sync with the live CI files).

For day-to-day development, build, and test instructions see
[`README.MD`](./README.MD) and [`AGENTS.md`](./AGENTS.md).

---

## 1. CI overview

The project has two parallel CI systems that talk to each other:

| System        | Config                                | Used for                                            |
| ------------- | ------------------------------------- | --------------------------------------------------- |
| **GitLab CI** | `.gitlab-ci.yml` (templates in `.gitlab/`) | Authoritative pipeline (tests, build, crates.io, GitLab release, GitHub mirror) |
| **GitHub Actions** | `.github/workflows/*.yml`         | Mirror-side: CI, build verification, Docker Hub image, GitHub release asset |

Pushing a tag to the GitLab repo is what actually triggers a release; the
GitHub side is reached via the `mirror:github` and `github:release:create`
GitLab jobs. Pushes to the `master` branch on GitHub run the GitHub
Actions CI for verification only — they do not publish.

---

## 2. Required secrets

All secrets below are **required** for a complete release. Missing secrets
will cause the corresponding job to fail or be silently skipped (e.g. the
Docker login step is guarded by `if [ -n "$DOCKERHUB_TOKEN" ]`).

### 2.1 GitLab project variables

Set in **GitLab → Settings → CI/CD → Variables**. Mark them **Protected**
(only available on protected branches/tags) and **Masked** (hidden from
job logs).

| Variable          | Required for      | Where to get it                                                                                          | Required scope                                           |
| ----------------- | ----------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| `CRATES_IO_TOKEN` | `release:crates-io` | <https://crates.io/settings/tokens>                                                                       | Token with the `publish` scope                           |
| `GITHUB_TOKEN`    | `mirror:github`, `github:release:create` | <https://github.com/settings/tokens> (fine-grained, classic also works)              | `Contents: Read and Write` on `hahihula/rendir`        |
| `DOCKERHUB_TOKEN` | `docker:build`, `docker:build:alpine`, `docker:build:using-template` | <https://hub.docker.com/settings/security>        | Docker Hub **Access Token** with `Read & Write`          |

`DOCKERHUB_USERNAME` is **not** a secret — it is hard-coded to
`hahihula` in `.gitlab-ci.yml` as a project variable. Override it in the
GitLab UI only if the Docker Hub account changes.

The following variables are provided automatically by the GitLab runner
and need no configuration: `CI_REGISTRY_IMAGE`, `CI_COMMIT_SHORT_SHA`,
`CI_COMMIT_TAG`, `CI_COMMIT_REF_NAME`, `CI_PROJECT_URL`, `CI_JOB_ID`.

### 2.2 GitHub repository secrets

Set in **GitHub → Settings → Secrets and variables → Actions →
Repository secrets**.

| Secret               | Required for                  | Where to get it                                                                                          | Required scope                                           |
| -------------------- | ----------------------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| `DOCKERHUB_USERNAME` | `Docker` workflow (login step) | Your Docker Hub username                                                                                  | —                                                        |
| `DOCKERHUB_TOKEN`    | `Docker` workflow (login step) | <https://hub.docker.com/settings/security>                                                                | Docker Hub **Access Token** with `Read & Write`          |

`GITHUB_TOKEN` for the GitHub release upload in `.github/workflows/ci.yml`
is **not** a secret you need to set — it is the temporary token GitHub
Actions provisions for every workflow run automatically. The `release`
job in `ci.yml` already requests `contents: write` permission.

The `build-multiple` job in the `Docker` workflow pushes multi-arch
images for `linux/amd64` and `linux/arm64`. It uses the same
`DOCKERHUB_USERNAME` / `DOCKERHUB_TOKEN` pair.

---

## 3. How to do a release

The canonical, step-by-step procedure lives in
[`RELEASE.md`](./RELEASE.md). The summary below is the happy path; the
long version includes pre-release checks, rollback, and verification.

### 3.1 Pre-flight

1. All required secrets from §2 are configured in both GitLab and
   GitHub.
2. `master` is green on the GitLab pipeline (last commit passes
   `test:linux:cli` and `build:linux:cli`).
3. Working tree is clean locally.

### 3.2 Bump and tag

```bash
git checkout master
git pull origin master
cargo test                                            # local sanity
$EDITOR rendir/Cargo.toml                             # bump `version`
git add rendir/Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"
git push origin master
```

Wait for the `master` pipeline to go green, then tag:

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

The tag **must** be annotated (`-a`) and match `v*` (e.g. `v0.2.0`).

### 3.3 Pipeline stages (in order)

After the tag is pushed, the GitLab pipeline runs these stages:

| Stage           | Jobs                                                  | Mode     |
| --------------- | ----------------------------------------------------- | -------- |
| `test`          | `test:linux:cli`                                      | auto     |
| `build`         | `build:linux:cli` (uploads `rendir-linux-x86_64`)     | auto     |
| `mirror`        | `mirror:github` (pushes tag/branch to GitHub)         | auto     |
| `github-release`| `github:release:create`                               | auto     |
| `release`       | `release:gitlab`                                      | auto     |
| `release-crate` | `release:crates-io`                                   | **manual** |
| `docker`        | `docker:build`, `docker:build:alpine`, `docker:build:using-template` | **manual** |

The **manual** jobs in `release-crate` and `docker` stages must be
triggered from the pipeline UI (the play button next to the job on the
**Pipelines** page). Run them in this order:

1. `release:crates-io` — publishes the `rendir` crate to crates.io.
2. `docker:build:using-template` — pushes canonical image
   `hahihula/rendir` with tags `$CI_COMMIT_TAG` and `latest`.
3. `docker:build` — pushes an additional `hahihula/rendir` image
   (built inline, does not use the shared template).
4. `docker:build:alpine` — pushes `hahihula/rendir-alpine` with tags
   `$CI_COMMIT_TAG` and `latest`.

### 3.4 GitHub side

The `mirror:github` job pushes the tag to `github.com/hahihula/rendir`,
which in turn triggers the GitHub Actions:

- `CI` workflow — builds, lints, tests, and (on tag) attaches
  `rendir` binary to a GitHub release via `softprops/action-gh-release@v1`.
- `Docker` workflow — builds and pushes multi-arch Docker images to
  Docker Hub as `hahihula/rendir` (tagged with the branch ref, PR
  ref, semver, and short SHA).

The `github:release:create` GitLab job ensures a matching GitHub release
exists (deleting and recreating if a stale one is there).

### 3.5 Verify

```bash
# crates.io
cargo search rendir
cargo install rendir --version X.Y.Z && rendir --version

# GitHub
git ls-remote --tags https://github.com/hahihula/rendir.git | grep vX.Y.Z
# https://github.com/hahihula/rendir/releases/tag/vX.Y.Z

# GitLab
# https://gitlab.com/hahihula/rendir/-/releases/vX.Y.Z

# Docker Hub
docker pull hahihula/rendir:X.Y.Z
docker pull hahihula/rendir-alpine:X.Y.Z
```

### 3.6 If something goes wrong

```bash
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
# fix, recommit, retag
```

Do not try to "patch" a broken release by re-pushing a corrected tag —
the `github:release:create` job deletes and recreates the release, and
the crates.io version is immutable.

---

## 4. Adding a new secret

1. Create the token in the upstream service (crates.io, Docker Hub,
   GitHub) with the minimum required scope.
2. In **GitLab → Settings → CI/CD → Variables**, add the variable,
   tick **Protect variable** and **Mask variable**, and (if the job
   runs on a tag) add the protected tag to the *Branches* protection
   list.
3. In **GitHub → Settings → Secrets and variables → Actions**, add a
   repository secret with the same name.
4. Reference the secret in the workflow file with
   `${{ secrets.NAME }}` (GitHub) or `$NAME` / `${NAME}` (GitLab).
5. Run a tag push to verify the job picks it up and the value is
   masked in the job log.

---

## 5. See also

- [`RELEASE.md`](./RELEASE.md) — full release procedure with rollback
  and post-release steps.
- [`AGENTS.md`](./AGENTS.md) — build/test commands, workspace layout.
- [`README.MD`](./README.MD) — project overview, install, usage.
- `.gitlab-ci.yml` / `.github/workflows/*.yml` — the source of truth
  for what the pipelines actually do.
