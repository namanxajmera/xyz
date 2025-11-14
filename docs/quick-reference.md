# Quick Reference: Package Manager Commands

This document lists the actual commands we'll need to execute for each package manager.

## Homebrew

```bash
# List installed packages (JSON)
brew list --json=v2

# List outdated packages
brew outdated

# Get package info
brew info <package> --json

# Update all packages
brew upgrade

# Update specific package
brew upgrade <package>

# Uninstall package
brew uninstall <package>
```

## npm

```bash
# List global packages (JSON)
npm list -g --json --depth=0

# List outdated global packages (JSON)
npm outdated -g --json

# Update global package
npm update -g <package>

# Update all global packages
npm update -g

# Uninstall global package
npm uninstall -g <package>
```

## yarn

```bash
# List global packages
yarn global list --json

# List outdated global packages
yarn global outdated --json

# Update global package
yarn global upgrade <package>

# Update all global packages
yarn global upgrade
```

## pnpm

```bash
# List global packages
pnpm list -g --depth=0 --json

# List outdated global packages
pnpm outdated -g --json

# Update global package
pnpm update -g <package>

# Update all global packages
pnpm update -g
```

## Cargo

```bash
# List installed binaries
cargo install --list

# Check for updates (requires cargo-install-update)
cargo install-update -l

# Update all
cargo install-update -a

# Update specific
cargo install-update <package>

# Uninstall
cargo uninstall <package>
```

## pip

```bash
# List installed packages (JSON)
pip list --format=json

# List outdated packages (JSON)
pip list --outdated --format=json

# Update package
pip install --upgrade <package>

# Update all (requires pip-review or manual)
pip list --outdated | cut -d ' ' -f1 | xargs -n1 pip install -U
```

## pipx

```bash
# List installed packages
pipx list --json

# Update all packages
pipx upgrade-all

# Update specific package
pipx upgrade <package>
```

## gem

```bash
# List installed gems
gem list

# List outdated gems
gem outdated

# Update gem
gem update <gem>

# Update all gems
gem update
```

## Go

```bash
# List installed binaries (check $GOPATH/bin or $GOBIN)
ls $GOPATH/bin
# or
ls $GOBIN

# Update binary
go install <package>@latest
```

## Composer

```bash
# List global packages
composer global show --format=json

# List outdated global packages
composer global outdated --format=json

# Update global package
composer global update <package>

# Update all global packages
composer global update
```

## Project Detection Patterns

### Node.js Projects
- `package.json` - npm/yarn/pnpm
- `package-lock.json` - npm
- `yarn.lock` - yarn
- `pnpm-lock.yaml` - pnpm

### Rust Projects
- `Cargo.toml` - Cargo
- `Cargo.lock` - Cargo (lock file)

### Python Projects
- `requirements.txt` - pip
- `pyproject.toml` - pip/poetry
- `Pipfile` - pipenv
- `Pipfile.lock` - pipenv
- `poetry.lock` - poetry

### Ruby Projects
- `Gemfile` - bundler
- `Gemfile.lock` - bundler

### Go Projects
- `go.mod` - Go modules
- `go.sum` - Go modules

### PHP Projects
- `composer.json` - Composer
- `composer.lock` - Composer

### Dart/Flutter Projects
- `pubspec.yaml` - pub

### Swift Projects
- `Package.swift` - Swift Package Manager

