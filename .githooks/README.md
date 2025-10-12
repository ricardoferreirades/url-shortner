# Git Hooks

This directory contains version-controlled git hooks for the project.

## Installation

Run the setup script to install hooks:

```bash
./.githooks/setup.sh
```

Or use the Make target:

```bash
make setup-hooks
```

## Available Hooks

### pre-push

Runs before every `git push` to ensure code quality:

1. **Format Check** - Verifies Rust and TOML files are formatted
2. **Linting** - Runs clippy with warnings as errors
3. **Spell Check** - Checks for typos in code and docs
4. **Type Check** - Ensures code compiles
5. **Tests** - Runs all test suites

**Note:** All checks must pass before push is allowed.

## Bypassing Hooks

Not recommended, but you can bypass with:

```bash
git push --no-verify
```

## Updating Hooks

After pulling updates that modify hooks, re-run:

```bash
./.githooks/setup.sh
```

