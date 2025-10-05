# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive code quality and accessibility audits
- CI/CD pipeline with multi-platform builds (Linux, macOS, Windows)
- Code coverage reporting with cargo-llvm-cov
- Enhanced README with project documentation and badges
- CHANGELOG.md for tracking project changes

### Changed
- Refactored `RateLimitError` to use `thiserror` for better error composition
- Updated all GitHub Actions to v4 (from deprecated v3)
- Enhanced error messages with proper Display implementations
- Improved TypeScript type safety by removing all `any` types

### Fixed
- **CI/CD**: Fixed deprecated GitHub Actions (v3 → v4)
- **CI/CD**: Updated WebKit package for Ubuntu 24.04 compatibility
- **CI/CD**: Added missing system dependencies for glib-sys build
- **Linting**: Resolved all 10 ESLint errors and 3 warnings
- **TypeScript**: Fixed `any` types in all visualization components
- **React**: Fixed useEffect dependency warnings
- **Security**: Fixed path exposure in error messages
- **Security**: Improved rate limiter type safety

## [0.1.0] - 2025-10-04

### Added
- Initial Tauri 2 + React 18 application setup
- SQLite database with connection pooling (DbPool)
- Transaction management with CSV import
- Debt payoff planning (Avalanche and Snowball strategies)
- Spending analytics with charts (pie, bar, line)
- Automatic transaction categorization with rules
- Dark mode support
- Comprehensive backend testing (60%+ coverage)
- Security features:
  - SQL injection prevention
  - Rate limiting on CSV imports
  - Error message sanitization
  - Input validation

### Backend Features
- **Commands**: 26 Tauri commands across 6 modules
- **Services**:
  - Transaction service with duplicate detection
  - CSV import service with streaming
  - Debt calculation service (avalanche/snowball)
  - Analytics service for spending insights
  - Categorization service with rule priority
- **Models**: 8 data models with proper SQLite integration
- **Database**: Migrations, seeding, and connection pooling

### Frontend Features
- **Pages**: Dashboard, Transactions, Debt Planner, Spending Analysis
- **Components**:
  - Transaction list with pagination
  - Debt management with payoff visualization
  - Spending charts (pie, bar, line)
  - CSV upload with column mapping
- **State Management**: Zustand stores for all major entities
- **UI**: Radix UI components with Tailwind CSS styling

### Testing
- Backend integration tests (56/68 passing)
- Contract tests for all commands
- Security tests for SQL injection prevention
- Frontend test infrastructure with Vitest

### Documentation
- SECURITY.md - Security best practices
- TESTING.md - Testing guidelines
- CLAUDE.md - Development conventions
- PR-REVIEW-RESPONSE.md - Code review feedback
- CODE_QUALITY_AUDIT.md - Quality assessment (Score: 7.6/10)
- ACCESSIBILITY_AUDIT.md - WCAG AA compliance review (Score: 5.5/10)

## Release Notes

### Known Issues
- **Accessibility**: Not yet WCAG AA compliant (see ACCESSIBILITY_AUDIT.md)
  - Missing form label associations
  - Color-only information without alternatives
  - No chart alternatives for screen readers
- **Frontend Tests**: Test infrastructure needs fixes
- **Clippy**: 2 minor warnings in Rust code

### Roadmap
- [ ] Achieve WCAG AA accessibility compliance (Tasks T182-T195)
- [ ] Fix remaining Clippy warnings (Task T011e)
- [ ] Implement missing contract tests for enhanced features
- [ ] Add account management UI (Tasks T119-T122)
- [ ] Add category management UI (Tasks T136-T141)
- [ ] Performance optimizations (virtualization, memoization)

---

## Changelog Guidelines

### Categories
- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security-related changes

### Version Format
- **Major** (x.0.0): Breaking changes
- **Minor** (0.x.0): New features, backward-compatible
- **Patch** (0.0.x): Bug fixes, backward-compatible

### Commit Message Format
Follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` - New feature (minor version bump)
- `fix:` - Bug fix (patch version bump)
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding/updating tests
- `chore:` - Maintenance tasks
- `ci:` - CI/CD changes

[Unreleased]: https://github.com/dwalleck/budget-balancer/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/dwalleck/budget-balancer/releases/tag/v0.1.0
