# Flatpak Build Guide

This directory contains the Flatpak packaging configuration for Database Structure Sync.

## Prerequisites

Install Flatpak and flatpak-builder:

```bash
# Ubuntu/Debian
sudo apt install flatpak flatpak-builder

# Fedora
sudo dnf install flatpak flatpak-builder

# Arch Linux
sudo pacman -S flatpak flatpak-builder
```

Add Flathub repository:

```bash
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
```

Install required runtimes and SDKs:

```bash
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//24.08
flatpak install flathub org.freedesktop.Sdk.Extension.node20//24.08
```

## Building

From the project root directory:

```bash
# Build the Flatpak
flatpak-builder --user --install --force-clean build-dir flatpak/com.dbstructsync.app.yml

# Or build without installing
flatpak-builder --force-clean build-dir flatpak/com.dbstructsync.app.yml
```

## Running

After installation:

```bash
flatpak run com.dbstructsync.app
```

## Creating a distributable bundle

```bash
# Build the repository
flatpak-builder --repo=repo --force-clean build-dir flatpak/com.dbstructsync.app.yml

# Create a single-file bundle
flatpak build-bundle repo database-structure-sync.flatpak com.dbstructsync.app
```

## Development Notes

### Permissions

The Flatpak is configured with the following permissions:

- `--share=network`: Required for database connections
- `--filesystem=home:ro`: Read-only access to home directory for SSH keys
- `--talk-name=org.freedesktop.secrets`: Access to system keyring for storing credentials

### Updating Dependencies

When updating Rust or Node.js dependencies, you may need to regenerate the sources:

```bash
# For Cargo dependencies
flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json

# For npm dependencies
flatpak-node-generator npm package-lock.json -o npm-sources.json
```

Note: For production builds, consider using generated source files instead of building from the internet to ensure reproducible builds.
