#!/usr/bin/env sh

# oat installer script
# Usage (recommended):
#   curl -fsSL https://raw.githubusercontent.com/paulartjomow/oat/main/scripts/install.sh | sh
# or with bash:
#   curl -fsSL https://raw.githubusercontent.com/paulartjomow/oat/main/scripts/install.sh | bash

set -euo pipefail

GITHUB_REPO="paulartjomow/oat"
BINARY_NAME="oat"

info() {
	printf "[oat-install] %s\n" "$*"
}

warn() {
	printf "[oat-install][warn] %s\n" "$*" >&2
}

err() {
	printf "[oat-install][error] %s\n" "$*" >&2
}

command_exists() {
	command -v "$1" >/dev/null 2>&1
}

detect_os() {
	case "$(uname -s)" in
		Linux) echo linux ;;
		Darwin) echo darwin ;;
		*) echo unknown ;;
	esac
}

detect_arch() {
	case "$(uname -m)" in
		x86_64|amd64) echo amd64 ;;
		arm64|aarch64) echo arm64 ;;
		*) echo unknown ;;
	esac
}

default_install_dir() {
	# Prefer /usr/local/bin if writable, otherwise ~/.local/bin
	if [ -w "/usr/local/bin" ]; then
		echo "/usr/local/bin"
	else
		echo "${HOME}/.local/bin"
	fi
}

ensure_dir() {
	if [ ! -d "$1" ]; then
		mkdir -p "$1"
	fi
}

download() {
	# args: url dest
	if command_exists curl; then
		curl -fsSL "$1" -o "$2"
	elif command_exists wget; then
		wget -q "$1" -O "$2"
	else
		err "Neither curl nor wget is installed"
		return 1
	fi
}

get_latest_release_json() {
	# Prints latest release JSON to stdout
	if command_exists curl; then
		curl -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
	elif command_exists wget; then
		wget -qO- "https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
	else
		err "Neither curl nor wget is installed"
		return 1
	fi
}

find_asset_url() {
	# args: release_json os arch
	# Attempts to find an asset matching common naming patterns
	release_json="$1"
	os="$2"
	arch="$3"

	# Common patterns we try, in order:
	# oat-<os>-<arch>.tar.gz
	# oat_<os>_<arch>.tar.gz
	# <BINARY_NAME>-<os>-<arch>
	# <BINARY_NAME>_<os>_<arch>

	# Minimize reliance on jq by using grep/sed
	# Extract all browser_download_url lines
	urls=$(printf "%s" "$release_json" | grep -o '"browser_download_url"[^"]*"[^"]*"' | sed 's/.*"browser_download_url"[^\"]*"\([^"]*\)".*/\1/')

	pattern1="${BINARY_NAME}-${os}-${arch}.tar.gz"
	pattern2="${BINARY_NAME}_${os}_${arch}.tar.gz"
	pattern3="${BINARY_NAME}-${os}-${arch}"
	pattern4="${BINARY_NAME}_${os}_${arch}"

	for p in "$pattern1" "$pattern2" "$pattern3" "$pattern4"; do
		match=$(printf "%s\n" "$urls" | grep "/$p$" || true)
		if [ -n "${match:-}" ]; then
			printf "%s" "$match"
			return 0
		fi
	done

	# Fallback: any asset that includes os and arch
	match=$(printf "%s\n" "$urls" | grep -i "$os" | grep -i "$arch" | head -n1 || true)
	if [ -n "${match:-}" ]; then
		printf "%s" "$match"
		return 0
	fi

	return 1

}

install_from_release() {
	inst_dir="$1"
	os="$2"
	arch="$3"

	info "Detecting latest release for ${os}/${arch}..."
	release_json=$(get_latest_release_json)

	asset_url=$(find_asset_url "$release_json" "$os" "$arch" || true)
	if [ -z "${asset_url:-}" ]; then
		return 1
	fi

	info "Downloading asset: $asset_url"
	tmpdir=$(mktemp -d)
	archive="$tmpdir/asset"
	download "$asset_url" "$archive"

	# Decide if archive or raw binary
	case "$asset_url" in
		*.tar.gz|*.tgz)
			info "Extracting archive"
			tar -xzf "$archive" -C "$tmpdir"
			# Try to find the binary
			candidate=$(find "$tmpdir" -type f -name "$BINARY_NAME" -perm +111 2>/dev/null | head -n1 || true)
			if [ -z "${candidate:-}" ]; then
				# Fallback: just any file named oat
				candidate=$(find "$tmpdir" -type f -name "$BINARY_NAME" 2>/dev/null | head -n1 || true)
			fi
			if [ -z "${candidate:-}" ]; then
				err "Could not find '${BINARY_NAME}' in the archive"
				return 1
			fi
			mv "$candidate" "$inst_dir/$BINARY_NAME"
			chmod +x "$inst_dir/$BINARY_NAME"
			;;
		*)
			info "Installing binary"
			mv "$archive" "$inst_dir/$BINARY_NAME"
			chmod +x "$inst_dir/$BINARY_NAME"
			;;
	esac

	info "Installed to $inst_dir/$BINARY_NAME"
}

install_with_cargo() {
	if ! command_exists cargo; then
		return 1
	fi
	info "Falling back to cargo install from git (this may take a while)..."
	cargo install --git "https://github.com/${GITHUB_REPO}.git" "$BINARY_NAME" --locked --force
}

main() {
	os=$(detect_os)
	arch=$(detect_arch)
	if [ "$os" = "unknown" ] || [ "$arch" = "unknown" ]; then
		err "Unsupported OS/arch: $(uname -s)/$(uname -m)"
		exit 1
	fi

	inst_dir="${INSTALL_DIR:-$(default_install_dir)}"
	ensure_dir "$inst_dir"

	if [ ! -w "$inst_dir" ]; then
		if command_exists sudo; then
			warn "Install directory '$inst_dir' not writable, retrying with sudo"
			export SUDO_ASKPASS=""
			# Re-run self with sudo to keep env
			exec sudo INSTALL_DIR="$inst_dir" sh "$0"
		else
			err "Install directory '$inst_dir' not writable and sudo not available"
			exit 1
		fi
	fi

	if install_from_release "$inst_dir" "$os" "$arch"; then
		info "oat installed successfully from release."
		exit 0
	fi

	warn "Could not find a prebuilt release asset for ${os}/${arch}."
	if install_with_cargo; then
		info "oat installed successfully via cargo."
		exit 0
	fi

	err "Installation failed. Ensure Rust is installed (https://rustup.rs) or use a supported platform."
	exit 1
}

main "$@"


