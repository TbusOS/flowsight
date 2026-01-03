# Contributing to FlowSight

Thank you for your interest in contributing to FlowSight! This document provides guidelines and instructions for contributing.

## 🚀 Getting Started

### Prerequisites

- Rust 1.75+
- Node.js 20+
- pnpm

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-username/flowsight.git
cd flowsight

# Install frontend dependencies
cd app
pnpm install
cd ..

# Build the project
cargo build --workspace

# Run tests
cargo test --workspace

# Run the desktop app in development mode
cd app
pnpm tauri dev
```

## 📝 Code Style

### Rust

- Follow the Rust style guide
- Use `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Write documentation for public APIs

### TypeScript

- Use TypeScript for all frontend code
- Follow the existing code style
- Use meaningful variable and function names

## 🔀 Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test --workspace`)
5. Run formatting (`cargo fmt --all`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### PR Guidelines

- Keep PRs focused on a single feature or fix
- Write clear commit messages
- Update documentation if needed
- Add tests for new features

## 🐛 Bug Reports

When filing a bug report, please include:

- FlowSight version
- Operating system
- Steps to reproduce
- Expected behavior
- Actual behavior
- Relevant logs or screenshots

## 💡 Feature Requests

We welcome feature requests! Please:

- Check if the feature is already requested
- Provide a clear use case
- Explain why this feature would be useful

## 📋 Areas for Contribution

### High Priority

- Language parser backends (C++, Rust, Java, Go)
- UI/UX improvements
- Performance optimization
- Documentation

### Medium Priority

- Additional async patterns
- Framework knowledge base entries
- Test coverage
- Accessibility improvements

### Exploration

- VSCode extension
- Web version
- Remote analysis server

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

## 🙏 Thank You!

Your contributions make FlowSight better for everyone. Thank you for helping!

