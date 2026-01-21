# Persona Prompt Manager

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.1.2-green.svg)
![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)
![Node](https://img.shields.io/badge/node-%3E%3D24.0.0-brightgreen.svg)
![Rust](https://img.shields.io/badge/rust-stable-orange.svg)
![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)

> A cross-platform desktop application built in Rust that enables digital artists and AI image generation practitioners to create, manage, and compose prompts for fictional character generation with consistency and precision.

---

## Table of Contents

- [Features](#features)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Development](#development)
  - [Available Scripts](#available-scripts)
  - [Project Structure](#project-structure)
- [Architecture](#architecture)
  - [Tech Stack](#tech-stack)
  - [Clean Architecture](#clean-architecture)
- [Configuration](#configuration)
- [Security](#security)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## Features

- **Persona Management** — Create and organize fictional character profiles with custom tags for easy categorization and retrieval.

- **Token Organization** — Structure prompt elements hierarchically across 7 granularity levels (Style, General, Hair, Face, Upper Body, Midsection, Lower Body) with positive/negative polarity support.

- **Prompt Composition** — Assemble complete prompts by selectively combining tokens from different granularity levels, with support for ad-hoc additions and weight modifiers.

- **Multi-Model Tokenization** — Accurate CLIP and T5 token counting for popular image generation models including Stable Diffusion XL, FLUX, AuraFlow, and DeepFloyd IF.

- **AI-Powered Generation** — Generate contextual token suggestions using multiple AI providers: OpenAI, Anthropic, Google, xAI, or Ollama (local).

- **Secure API Key Storage** — API keys are stored securely using OS-native credential storage (macOS Keychain, Windows Credential Manager, Linux Secret Service).

- **Import/Export** — Backup and restore all personas and tokens in JSON format for sharing or migration.

- **Cross-Platform** — Native desktop application for Windows, macOS, and Linux built with Tauri.

---

## Getting Started

### Prerequisites

- **Node.js** >= 24.0.0
- **Rust** (latest stable via [rustup](https://rustup.rs/))
- **Platform-specific dependencies:**

  <details>
  <summary><strong>Linux (Debian/Ubuntu)</strong></summary>

  ```bash
  sudo apt update
  sudo apt install -y \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf
  ```

  </details>

  <details>
  <summary><strong>Linux (Fedora)</strong></summary>

  ```bash
  sudo dnf install -y \
    webkit2gtk4.1-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel
  ```

  </details>

  <details>
  <summary><strong>Linux (Arch)</strong></summary>

  ```bash
  sudo pacman -S --needed \
    webkit2gtk-4.1 \
    libayatana-appindicator \
    librsvg
  ```

  </details>

  <details>
  <summary><strong>macOS</strong></summary>

  Install Xcode Command Line Tools:

  ```bash
  xcode-select --install
  ```

  </details>

  <details>
  <summary><strong>Windows</strong></summary>

  Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload.

  </details>

### Installation

```bash
# Clone the repository
git clone https://github.com/j-about/Persona-Prompt-Manager.git
cd Persona-Prompt-Manager

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

Production builds are output to `src-tauri/target/release/bundle/`.

---

## Development

### Available Scripts

| Command               | Description                              |
| --------------------- | ---------------------------------------- |
| `npm run dev`         | Start Vite development server (frontend) |
| `npm run build`       | Build frontend for production            |
| `npm run check`       | Type-check Svelte components             |
| `npm run lint`        | Run ESLint                               |
| `npm run format`      | Format code with Prettier                |
| `npm run validate`    | Run all checks (lint + check + format)   |
| `npm run tauri dev`   | Start full Tauri development application |
| `npm run tauri build` | Build production desktop application     |

### Project Structure

```
Persona-Prompt-Manager/
├── src/                        # SvelteKit frontend
│   ├── app.css                 # Global styles (Tailwind)
│   ├── app.html                # HTML template
│   ├── lib/
│   │   ├── components/         # Reusable UI components
│   │   │   ├── persona/        # Persona-specific components
│   │   │   ├── token/          # Token management components
│   │   │   └── ui/             # Generic UI primitives
│   │   ├── services/           # Tauri IPC service layer
│   │   ├── stores/             # Svelte 5 reactive stores
│   │   └── types/              # TypeScript type definitions
│   └── routes/                 # SvelteKit pages
│       ├── +layout.svelte      # Root layout with navigation
│       ├── +page.svelte        # Dashboard
│       ├── personas/           # Persona CRUD pages
│       ├── compose/            # Prompt composition
│       ├── settings/           # Configuration
│       └── about/              # About page
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── commands/           # Tauri IPC command handlers
│   │   ├── domain/             # Business logic & entities
│   │   ├── infrastructure/     # Database, AI, keyring adapters
│   │   ├── error/              # Error handling
│   │   ├── lib.rs              # Library entry point
│   │   └── main.rs             # Application entry point
│   ├── icons/                  # Application icons
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
├── static/                     # Static assets
├── package.json                # Node dependencies & scripts
├── tsconfig.json               # TypeScript configuration
├── vite.config.ts              # Vite build configuration
├── svelte.config.js            # SvelteKit configuration
├── eslint.config.js            # ESLint rules
└── prettier.config.js          # Prettier formatting rules
```

---

## Architecture

### Tech Stack

| Layer            | Technology                                                       |
| ---------------- | ---------------------------------------------------------------- |
| **Frontend**     | SvelteKit 2.0, Svelte 5, TypeScript 5, Tailwind CSS 4, DaisyUI 5 |
| **Backend**      | Rust, Tauri 2, SQLite (WAL mode)                                 |
| **AI**           | genai crate (OpenAI, Anthropic, Google, xAI, Ollama)             |
| **Tokenization** | HuggingFace Tokenizers (CLIP, T5)                                |
| **Security**     | OS-native keyring (keyring crate)                                |

### Clean Architecture

The Rust backend follows clean architecture principles with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (Svelte)                       │
│   Routes  →  Stores  →  Services  →  Tauri IPC              │
└─────────────────────────────────────────────────────────────┘
                            │
                      Tauri Bridge
                            │
┌─────────────────────────────────────────────────────────────┐
│                      Backend (Rust)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   Commands   │──│    Domain    │──│  Infrastructure  │   │
│  │  (IPC Layer) │  │   (Logic)    │  │  (DB, AI, Keys)  │   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Layers:**

- **Commands** — Thin IPC handlers that expose functionality to the frontend via Tauri's invoke system.
- **Domain** — Core business entities (Persona, Token, Prompt) and logic, independent of external services.
- **Infrastructure** — External adapters for SQLite database, AI providers, and OS keyring.

---

## Configuration

### AI Provider Setup

1. Navigate to **Settings** in the application
2. Select your preferred AI provider (OpenAI, Anthropic, Google, xAI, or Ollama)
3. Enter your API key (stored securely in your OS keyring)
4. Configure per-persona AI settings in the persona editor

### Supported AI Providers

| Provider  | Default Model           | API Key Required |
| --------- | ----------------------- | ---------------- |
| OpenAI    | gpt-5.2-pro             | Yes              |
| Anthropic | claude-opus-4-5         | Yes              |
| Google    | gemini-3-pro-preview    | Yes              |
| xAI       | grok-4-1-fast-reasoning | Yes              |
| Ollama    | llama3.2                | No (local)       |

### Import/Export

- **Export**: Settings → Export All Personas → Downloads a JSON file
- **Import**: Settings → Import Personas → Select JSON file → Choose conflict resolution strategy

---

## Security

This application prioritizes security through multiple layers:

- **No unsafe Rust code** — Enforced via `#![forbid(unsafe_code)]` compiler directive.

- **OS-native credential storage** — API keys are stored in platform-specific secure storage:
  - macOS: Keychain
  - Windows: Credential Manager
  - Linux: Secret Service (libsecret)

- **Content Security Policy** — Strict CSP headers prevent XSS and other injection attacks.

- **Type safety** — Full type checking across both TypeScript (frontend) and Rust (backend) codebases.

- **No telemetry** — The application does not collect or transmit any user data.

---

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- [Tauri](https://tauri.app/) — Cross-platform desktop framework
- [SvelteKit](https://kit.svelte.dev/) — Frontend framework
- [Svelte 5](https://svelte.dev/) — Reactive UI library
- [DaisyUI](https://daisyui.com/) — Tailwind CSS component library
- [Tailwind CSS](https://tailwindcss.com/) — Utility-first CSS framework
- [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers) — Fast tokenization library
- [genai](https://github.com/jeremychone/rust-genai) — Multi-provider AI integration for Rust
- [rusqlite](https://github.com/rusqlite/rusqlite) — SQLite bindings for Rust
- [keyring](https://github.com/hwchen/keyring-rs) — Cross-platform credential storage

---

<p align="center">
  Made with Rust and Svelte
</p>
