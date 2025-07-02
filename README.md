# Pandoc Desktop

<div align="center">

![Pandoc Desktop Logo](public/pandoc-banner.svg)

**A modern, cross-platform desktop application for universal document conversion powered by Pandoc**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://tauri.app/)
[![Vue.js](https://img.shields.io/badge/Vue.js-3.0-green.svg)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-blue.svg)](https://www.typescriptlang.org/)

[Download](#installation) • [Features](#features) • [Documentation](#usage) • [Contributing](#contributing)

</div>

## About This Project

Pandoc Desktop is a modern, user-friendly desktop application that provides a graphical interface for [Pandoc](https://pandoc.org/), the universal document converter. Built with Tauri, Vue.js, and TypeScript, it combines the power of Pandoc with an intuitive desktop experience.

**🚀 Ready to convert documents?** Download the latest release and start converting instantly.

**👨‍💻 Developer?** Welcome! Check out our [development setup](#development-setup) to get started.

## What is Pandoc Desktop?

Pandoc Desktop bridges the gap between Pandoc's powerful command-line capabilities and everyday usability by providing:

- 🖥️ **Cross-Platform Desktop App**: Native performance on Windows, macOS, and Linux
- 📄 **Universal Document Conversion**: Support for 40+ input formats and 60+ output formats
- 🎛️ **Intuitive Interface**: Drag-and-drop file handling with visual configuration
- ⚡ **Built-in Pandoc Management**: Automatic Pandoc installation and version management
- 🎨 **Modern Design**: Clean, responsive UI with dark/light theme support
- 🔧 **Advanced Configuration**: PDF engine selection, custom options, and batch processing
- 📦 **Portable Mode**: Self-contained installation with bundled dependencies

## Features

### 📚 **Comprehensive Format Support**

- **Input Formats**: Markdown, HTML, LaTeX, DocBook, EPUB, ODT, DOCX, and many more
- **Output Formats**: PDF, HTML, LaTeX, Word, PowerPoint, EPUB, and 80+ other formats
- **Smart Detection**: Automatic input format detection based on file extension

### 🎯 **User-Focused Design**

- **Drag & Drop**: Simply drag files into the application to start converting
- **Visual Configuration**: No command-line knowledge required
- **Real-time Preview**: See conversion options and settings before processing
- **Batch Processing**: Convert multiple files simultaneously
- **Progress Tracking**: Visual feedback during conversion process

### 🔧 **Advanced Capabilities**

- **PDF Engine Selection**: Choose from multiple PDF engines (Typst, LaTeX, wkhtmltopdf, etc.)
- **Custom Pandoc Options**: Fine-tune conversion with advanced parameters
- **Template Support**: Use custom templates for consistent output styling
- **Resource Management**: Automatic handling of images and linked resources

### 🛠️ **Development Features**

- **Bundled Dependencies**: Includes Pandoc and PDF engines for offline use
- **Auto-updating**: Keep Pandoc and tools up-to-date automatically
- **Cross-platform**: Single codebase for Windows, macOS, and Linux
- **Modern Architecture**: Built with Tauri 2.0, Vue 3, and TypeScript

## Installation

### Download Pre-built Binaries

Visit our [Releases page](https://github.com/DemoMacro/pandoc-desktop/releases) to download the latest version for your platform:

- **Windows**: `pandoc-desktop-x.x.x-setup.exe`
- **macOS**: `pandoc-desktop-x.x.x.dmg`
- **Linux**: `pandoc-desktop-x.x.x.AppImage` or `.deb`/.rpm` packages

### Package Managers

```bash
# Windows (Chocolatey)
choco install pandoc-desktop

# macOS (Homebrew)
brew install --cask pandoc-desktop

# Linux (Snap)
sudo snap install pandoc-desktop
```

## Usage

### Quick Start

1. **Launch** Pandoc Desktop
2. **Select Input**: Drag & drop a file or click "Select File"
3. **Choose Output Format**: Pick from the format dropdown
4. **Configure Options**: Set output location and any special settings
5. **Convert**: Click the "Convert" button and wait for completion

### Supported Formats

#### Input Formats

```
Markdown, HTML, LaTeX, DocBook, EPUB, ODT, DOCX, RST,
Textile, MediaWiki, DokuWiki, Creole, JATS, TEI, Typst,
Jupyter Notebooks, CSV, TSV, JSON, YAML, BibTeX,
Org Mode, Muse, RIS, OPML, FB2, and 40+ more...
```

#### Output Formats

```
PDF, HTML, LaTeX, Word (DOCX), PowerPoint (PPTX),
EPUB, ODT, RTF, Plain Text, Slides (reveal.js, Beamer),
MediaWiki, DokuWiki, AsciiDoc, Typst, JATS, TEI,
JSON, BibTeX, Groff, Man Pages, and 60+ other formats
```

### PDF Conversion

Pandoc Desktop includes multiple PDF engines for high-quality document conversion:

- **Typst** (recommended): Modern, fast PDF generation
- **LaTeX** (pdflatex, xelatex, lualatex): Academic publishing standard
- **wkhtmltopdf**: HTML-to-PDF conversion
- **weasyprint**: CSS-based PDF generation

## Development Setup

### Prerequisites

- **Node.js** 18.x or higher
- **Rust** 1.70.0 or higher
- **pnpm** 8.x or higher (recommended package manager)
- **Git** for version control

### Getting Started

1. **Clone the repository**:

```bash
git clone https://github.com/DemoMacro/pandoc-desktop.git
cd pandoc-desktop
```

2. **Install dependencies**:

```bash
pnpm install
```

3. **Start development server**:

```bash
pnpm tauri dev
```

4. **Build for production**:

```bash
pnpm tauri build
```

### Development Commands

```bash
pnpm dev              # Start development server
pnpm build            # Build frontend for production
pnpm tauri dev        # Start Tauri development mode
pnpm tauri build      # Build native application
pnpm lint             # Run code linting
pnpm format           # Format code with prettier
```

## Project Structure

```
pandoc-desktop/
├── src/                         # Vue.js frontend source
│   ├── components/              # Vue components
│   │   ├── FileInput.vue        # File selection interface
│   │   ├── FormatSelector.vue   # Format selection dropdown
│   │   ├── ConvertButton.vue    # Conversion trigger
│   │   ├── OutputConfig.vue     # Output configuration
│   │   ├── PandocManager.vue    # Pandoc management UI
│   │   └── SettingsPanel.vue    # Application settings
│   ├── composables/             # Vue composition functions
│   │   ├── useConversion.ts     # Conversion logic
│   │   ├── usePandocManager.ts  # Pandoc management
│   │   ├── useFileHandling.ts   # File operations
│   │   └── useUI.ts             # UI state management
│   ├── types/                   # TypeScript definitions
│   └── App.vue                  # Main application component
├── src-tauri/                   # Tauri Rust backend
│   ├── src/
│   │   ├── lib.rs               # Main Tauri application
│   │   ├── pandoc.rs            # Pandoc integration
│   │   ├── manager.rs           # Tool management
│   │   └── utils.rs             # Utility functions
│   ├── capabilities/            # Tauri capabilities
│   ├── resources/               # Bundled tools (Pandoc, Typst)
│   └── tauri.conf.json          # Tauri configuration
├── public/                      # Static assets and logos
├── package.json                 # Frontend dependencies
└── README.md                    # This file
```

## Architecture & Design

### 🏗️ **Technology Stack**

- **Frontend**: Vue.js 3 with Composition API and TypeScript
- **Backend**: Tauri 2.0 with Rust for native performance
- **UI Framework**: Pico CSS for clean, semantic styling
- **Build System**: Vite for fast development and optimized builds
- **Package Manager**: pnpm for efficient dependency management

### 🔧 **Core Principles**

1. **User-First Design**: Prioritize ease of use over technical complexity
2. **Cross-Platform Compatibility**: Single codebase, native performance
3. **Offline Capability**: Bundle dependencies for reliable offline use
4. **Modern Architecture**: Leverage latest web and native technologies
5. **Extensible Design**: Modular structure for easy feature additions

### 📦 **Bundled Tools**

Pandoc Desktop includes pre-compiled tools for immediate use:

- **Pandoc**: Latest stable version with full format support
- **Typst**: Modern PDF generation engine
- **LaTeX Tools**: When available on the system

## Configuration

### Application Settings

Settings are stored in platform-specific locations:

- **Windows**: `%APPDATA%/pandoc-desktop/`
- **macOS**: `~/Library/Application Support/pandoc-desktop/`
- **Linux**: `~/.config/pandoc-desktop/`

### Custom Pandoc Options

Advanced users can specify custom Pandoc command-line options through the settings panel for specialized conversion needs.

## Contributing

We welcome contributions! Here's how to get started:

### Quick Setup

```bash
# Clone and setup
git clone https://github.com/DemoMacro/pandoc-desktop.git
cd pandoc-desktop
pnpm install

# Start development
pnpm tauri dev
```

### Development Workflow

1. **Code**: Follow our TypeScript and Vue.js best practices
2. **Test**: Ensure conversions work across different formats
3. **Lint**: Run `pnpm lint` to check code quality
4. **Commit**: Use conventional commits (`feat:`, `fix:`, `docs:`, etc.)
5. **Submit**: Create a Pull Request with clear description

### Areas for Contribution

- 🐛 **Bug Reports**: Help us identify and fix issues
- ✨ **Feature Requests**: Suggest new functionality
- 📚 **Documentation**: Improve guides and examples
- 🌍 **Internationalization**: Add support for more languages
- 🎨 **UI/UX**: Enhance the user interface and experience

## Troubleshooting

### Common Issues

**PDF conversion fails**

- Ensure a PDF engine is installed (bundled Typst is recommended)
- Check input file format compatibility
- Verify sufficient disk space for output

**Application won't start**

- Check minimum system requirements
- Try running as administrator (Windows) or with appropriate permissions
- Clear application cache and restart

**Conversion takes too long**

- Large files may require more time
- Consider using a faster PDF engine like Typst
- Close other resource-intensive applications

### Getting Help

- 📫 [Report Issues](https://github.com/DemoMacro/pandoc-desktop/issues)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **[Pandoc](https://pandoc.org/)**: The universal document converter that powers this application
- **[Tauri](https://tauri.app/)**: For the excellent cross-platform framework
- **[Vue.js](https://vuejs.org/)**: For the reactive frontend framework
- **[Typst](https://typst.app/)**: For modern PDF generation capabilities

---

<div align="center">

**Built with ❤️ using modern web technologies**

[⭐ Star us on GitHub](https://github.com/DemoMacro/pandoc-desktop) • [🐛 Report Bug](https://github.com/DemoMacro/pandoc-desktop/issues) • [💡 Request Feature](https://github.com/DemoMacro/pandoc-desktop/issues)

</div>
