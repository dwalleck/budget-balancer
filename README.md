# Budget Balancer

[![CI/CD Pipeline](https://github.com/dwalleck/budget-balancer/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/dwalleck/budget-balancer/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A personal finance management application for tracking transactions, managing debt payoff strategies, and analyzing spending patterns.

## Features

- 📊 **Transaction Management**: Import and categorize transactions from CSV files
- 💳 **Debt Payoff Planning**: Avalanche and Snowball strategies with detailed payment schedules
- 📈 **Spending Analytics**: Visualize spending patterns by category over time
- 🎯 **Spending Targets**: Set and track monthly spending goals
- 🔄 **Automatic Categorization**: Rule-based transaction categorization
- 🌙 **Dark Mode**: Full dark mode support

## Tech Stack

- **Frontend**: React 18, TypeScript, Tailwind CSS, Radix UI, Recharts
- **Backend**: Rust, Tauri 2, SQLite
- **State Management**: Zustand
- **Testing**: Vitest (frontend), Cargo (backend)
- **Build Tool**: Vite

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/) (latest)
- [Rust](https://rustup.rs/) (stable)
- System dependencies (see [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/))

### Development

```bash
# Install dependencies
bun install

# Run development server
bun run tauri dev

# Run tests
bun test                    # Frontend tests
cd src-tauri && cargo test  # Backend tests

# Linting
bun run lint                # TypeScript/React
cd src-tauri && cargo clippy  # Rust
```

### Building

```bash
# Build for production
bun run tauri build
```

## Documentation

- [Code Quality Audit](CODE_QUALITY_AUDIT.md) - Code quality analysis and recommendations
- [Accessibility Audit](ACCESSIBILITY_AUDIT.md) - WCAG AA compliance assessment
- [Security Guidelines](SECURITY.md) - Security best practices
- [Testing Guide](TESTING.md) - Comprehensive testing documentation
- [Development Guidelines](CLAUDE.md) - Project conventions and structure

## Project Structure

```
budget-balancer/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── pages/             # Application pages
│   ├── stores/            # Zustand state stores
│   └── lib/               # Utilities and helpers
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── commands/      # Tauri command handlers
│   │   ├── services/      # Business logic
│   │   ├── models/        # Data models
│   │   └── db/            # Database layer
│   └── tests/             # Backend tests
└── specs/                 # Feature specifications
```

## Contributing

1. Follow the conventions in [CLAUDE.md](CLAUDE.md)
2. Write tests for new features
3. Ensure linting passes: `bun run lint && cd src-tauri && cargo clippy`
4. Maintain WCAG AA accessibility compliance

## License

MIT

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
