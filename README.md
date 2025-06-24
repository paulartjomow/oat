
![Logo](https://www.prixix.com/wp-content/uploads/2024/06/oat_logo_wide.png)

# oat (Open Actions Toolbox)

The CLI toolbox for ~~OpenAI~~ everything

## Features

- **Hash Generation**: Generate MD5, SHA256, and other hash types
- **Password Generation**: Create secure passwords with various options
- **Currency Conversion**: Real-time currency conversion rates
- **Auto-Update System**: Automatically check and install updates from GitHub

## Installation

### From GitHub Releases (Recommended)

Download the latest binary for your platform from the [releases page](https://github.com/Prixix/oat/releases).

### From Source

```bash
git clone https://github.com/Prixix/oat.git
cd oat
cargo build --release
```

## Usage

### Available Commands

```bash
oat generate    # Generate passwords and other data
oat hash        # Generate hashes for text or files
oat currency    # Convert between currencies
oat update      # Check for and install updates
```

### Auto-Update System

The oat CLI includes a sophisticated auto-update system that keeps your installation current with the latest features and security updates.

#### Automatic Update Checking

- **Background Checks**: oat automatically checks for updates once per day when you run any command
- **Non-Intrusive**: Update checks run silently in the background and won't slow down your commands
- **Smart Timing**: Uses a timestamp file (`~/.oat_last_update_check`) to avoid excessive API calls

#### Manual Update Commands

```bash
# Check if updates are available
oat update --check-only
oat update -c

# Install the latest version
oat update
```

#### Update Process

1. **Version Comparison**: Uses semantic versioning to compare your current version with the latest GitHub release
2. **User Confirmation**: Always asks for confirmation before installing updates
3. **Release Notes**: Shows you what's new in the latest version
4. **Cross-Platform**: Automatically detects your platform and downloads the correct binary
5. **Safe Installation**: Uses the `self_update` crate for reliable binary replacement

#### Supported Platforms

- **Linux**: x86_64, aarch64
- **macOS**: Intel (x86_64), Apple Silicon (aarch64)
- **Windows**: x86_64

#### Configuration

You can disable automatic update checking by setting an environment variable:

```bash
export OAT_AUTO_UPDATE_CHECK=false
```

#### For Developers

##### Creating a New Release

Use the included release script to automatically bump versions and create releases:

```bash
# Patch release (0.1.0 -> 0.1.1)
./scripts/release.sh patch

# Minor release (0.1.0 -> 0.2.0)
./scripts/release.sh minor

# Major release (0.1.0 -> 1.0.0)
./scripts/release.sh major
```

The script will:
1. Update the version in `Cargo.toml`
2. Update `Cargo.lock`
3. Commit the changes
4. Create and push a git tag
5. Trigger GitHub Actions to build and publish the release

##### GitHub Actions Workflow

The project includes a comprehensive GitHub Actions workflow (`.github/workflows/release.yml`) that:

- Builds binaries for all supported platforms
- Creates compressed archives for distribution
- Publishes releases with detailed release notes
- Handles cross-compilation for different architectures

## TODO
