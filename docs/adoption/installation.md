# Installation Guide

Multiple ways to install Sruja DSL based on your platform and preferences.

## Quick Install

### macOS / Linux

```bash
curl -sSL https://sruja.dev/install.sh | bash
```

This script:
- Detects your OS and architecture
- Downloads the latest release
- Installs to `/usr/local/bin/sruja`
- Adds to PATH

### Windows

```powershell
# PowerShell
iwr https://sruja.dev/install.ps1 | iex

# Or download from releases
# https://github.com/sruja-ai/sruja/releases
```

---

## Package Managers

### Homebrew (macOS)

```bash
brew tap sruja-ai/sruja
brew install sruja
```

### npm / yarn

```bash
npm install -g sruja
# or
yarn global add sruja
```

### Go Install

```bash
go install github.com/sruja-ai/sruja/apps/cli/cmd@latest
```

### Docker

```bash
docker pull sruja/sruja:latest
docker run --rm -v $(pwd):/workspace sruja/sruja compile architecture.sruja
```

---

## Manual Installation

### Download Binary

1. Visit [Releases](https://github.com/sruja-ai/sruja/releases)
2. Download for your platform:
   - `sruja-darwin-amd64` (macOS Intel)
   - `sruja-darwin-arm64` (macOS Apple Silicon)
   - `sruja-linux-amd64` (Linux 64-bit)
   - `sruja-linux-arm64` (Linux ARM)
   - `sruja-windows-amd64.exe` (Windows)

3. Make executable (Unix):
   ```bash
   chmod +x sruja-darwin-amd64
   mv sruja-darwin-amd64 /usr/local/bin/sruja
   ```

4. Add to PATH (Windows):
   - Add the directory containing `sruja.exe` to your PATH

### Build from Source

```bash
git clone https://github.com/sruja-ai/sruja.git
cd sruja
go build -o sruja ./apps/cli/cmd
sudo mv sruja /usr/local/bin/
```

---

## Verify Installation

```bash
sruja --version
```

You should see output like:
```
sruja version 1.0.0
```

---

## Editor Integration

### VSCode

1. Open Extensions (Cmd/Ctrl + Shift + X)
2. Search for "Sruja"
3. Click Install
4. Open any `.sruja` file

### Vim / Neovim

Using [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig):

```lua
require('lspconfig').sruja.setup{}
```

### IntelliJ / JetBrains

Plugin coming soon. For now, use LSP client.

---

## Troubleshooting

### Command Not Found

Make sure `sruja` is in your PATH:

```bash
# Check if installed
which sruja

# Add to PATH (Unix)
export PATH=$PATH:/usr/local/bin

# Or add to shell config (.bashrc, .zshrc, etc.)
echo 'export PATH=$PATH:/usr/local/bin' >> ~/.zshrc
```

### Permission Denied

```bash
chmod +x /usr/local/bin/sruja
```

### Version Issues

Update to latest:

```bash
# Homebrew
brew upgrade sruja

# npm
npm update -g sruja

# Go
go install github.com/sruja-ai/sruja/apps/cli/cmd@latest
```

---

## Next Steps

After installation:

1. Run `sruja init` to create your first project
2. Read the [Quick Start Guide](./quickstart.md)
3. Try the [Examples](../examples/README.md)

---

## Platform-Specific Notes

### macOS

- Apple Silicon (M1/M2/M3): Use `-arm64` binary
- Intel: Use `-amd64` binary

### Linux

- Most distributions: Use `-amd64` binary
- ARM servers (Raspberry Pi, AWS Graviton): Use `-arm64` binary

### Windows

- Use `.exe` binary
- May need to add exception in Windows Defender
- WSL: Use Linux binary instead

---

## Uninstall

### Homebrew

```bash
brew uninstall sruja
```

### npm

```bash
npm uninstall -g sruja
```

### Manual

```bash
# Remove binary
rm /usr/local/bin/sruja

# Remove config (optional)
rm -rf ~/.sruja
```

