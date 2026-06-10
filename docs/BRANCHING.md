# Branching strategy

This project uses a lightweight Git Flow adapted for a small desktop app.

## Branches

| Branch | Purpose | Deploy / release |
|--------|---------|------------------|
| **`main`** | **Production.** Stable, tagged releases only. | Ship binaries from here. |
| **`dev`** | **Integration.** Active development merges here first. | Preview / internal builds. |
| **`feature/*`** | **Working branches.** One feature or fix per branch. | Never deploy directly. |

> **`main` is production.** We do not maintain a separate `prod` branch — tags on `main` (e.g. `v0.2.0`) mark releases.

## Workflow

```
feature/settings-view ──PR──► dev ──PR──► main ──tag──► v0.x.x
feature/stop-streaming ──PR──► dev
```

### 1. Start work

```bash
git checkout dev
git pull origin dev
git checkout -b feature/your-feature-name
```

### 2. Commit

Use clear, imperative messages:

```bash
git add .
git commit -m "Add settings panel for Ollama host and context size"
```

### 3. Open PR → `dev`

- Target branch: **`dev`**
- Link related issue if any
- Ensure `npm run check` passes locally

### 4. Release → `main`

When `dev` is stable:

```bash
git checkout main
git merge dev
# Update CHANGELOG.md, package.json, Cargo.toml version
git tag -a v0.2.0 -m "v0.2.0"
git push origin main --tags
git checkout dev
git merge main   # keep dev in sync
git push origin dev
```

## Rules

- **Never force-push `main`**
- **No direct commits to `main`** except release merges (use PRs from `dev`)
- **Delete feature branches** after merge
- **Tag every release** on `main`
- **Update `CHANGELOG.md`** before tagging

## Versioning

[Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`

- **PATCH** — bug fixes
- **MINOR** — new features (backward compatible)
- **MAJOR** — breaking changes

Keep in sync: `package.json`, `src-tauri/Cargo.toml`, `CHANGELOG.md`, git tag.
