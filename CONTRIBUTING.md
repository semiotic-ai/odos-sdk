# Contributing to odos-sdk

Thank you for your interest in contributing to odos-sdk! This document provides guidelines and instructions for contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.90 or later
- Git
- (Optional) cargo-audit for security audits

### Clone the Repository

```bash
git clone https://github.com/semiotic-ai/odos-rs.git
cd odos-rs
```

### Build the Project

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

## Development Workflow

### Before You Start

1. Check existing issues and pull requests to avoid duplicate work
2. For significant changes, open an issue first to discuss the approach
3. Fork the repository and create a feature branch from `main`

### Making Changes

1. **Write Code**: Make your changes following the code style guidelines below
2. **Add Tests**: Ensure your changes are covered by tests
3. **Update Documentation**: Update relevant documentation and examples
4. **Run Quality Checks**: Execute all checks locally before pushing

### Quality Checks

Run these commands before committing:

```bash
# Format code
cargo fmt

# Run linter with warnings as errors
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-targets --all-features

# Run doc tests
cargo test --doc

# Check documentation builds without warnings
cargo doc --no-deps

# (Optional) Security audit
cargo audit
```

### Commit Messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/). Commit messages are validated by CI.

Format: `<type>(<scope>): <description>`

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Test additions or changes
- `chore`: Maintenance tasks
- `ci`: CI/CD changes
- `perf`: Performance improvements

**Breaking Changes:**
- Use `!` after type/scope: `feat!: remove deprecated API`
- Or use `BREAKING CHANGE:` in the commit body

**Examples:**
```
feat(client): add custom timeout configuration
fix(error): correct error code for rate limits
docs(readme): update installation instructions
refactor!: remove deprecated EndpointBase type
```

### Pull Request Process

1. **Update CHANGELOG.md**: Add your changes under `[Unreleased]`
2. **Ensure CI Passes**: All automated checks must pass
3. **Request Review**: Tag maintainers for review
4. **Address Feedback**: Respond to review comments and make necessary changes
5. **Squash Commits**: We use squash-and-merge for clean history

## Code Style Guidelines

### Rust Style

- Follow standard Rust formatting (`cargo fmt`)
- Use clippy recommendations (`cargo clippy`)
- Prefer named string interpolation over positional
- Use `dotenvy` instead of `dotenv` for environment variables

### Documentation

- Document all public items with doc comments
- Include examples in doc comments where appropriate
- Examples should be compilable or marked with appropriate directives (`no_run`, `ignore`, etc.)
- Focus on user-facing documentation, not implementation details

### Testing

- Write unit tests for all new functionality
- Test the code we write, not established dependencies
- Use descriptive test names that explain what is being tested
- Include edge cases and error conditions

### Error Handling

- Use `Result` for fallible operations
- Provide context with error messages
- Use `thiserror` for custom error types
- Document error conditions in function documentation

### Dependencies

- Minimize new dependencies
- Prefer well-maintained crates
- Document rationale for new dependencies in `DEPENDENCIES.md`
- Use feature flags for optional functionality

## Project Structure

```
odos-rs/
├── src/
│   ├── lib.rs          # Public API re-exports
│   ├── api.rs          # API types and requests
│   ├── client.rs       # HTTP client and retry logic
│   ├── error.rs        # Error types
│   ├── sor.rs          # Smart Order Router client
│   ├── chain.rs        # Chain support traits
│   ├── contract.rs     # Contract addresses
│   └── ...
├── abis/               # Contract ABIs
├── tests/              # Integration tests (if any)
├── DEPENDENCIES.md     # Dependency documentation
├── CHANGELOG.md        # Version history
└── README.md           # User documentation
```

## Release Process

This project follows [Semantic Versioning](https://semver.org/).

### Version Numbers

- **MAJOR**: Breaking changes (incompatible API changes)
- **MINOR**: New features (backward-compatible)
- **PATCH**: Bug fixes (backward-compatible)

### Pre-1.0 Versioning

- Breaking changes increment MINOR version
- New features increment MINOR version
- Bug fixes increment PATCH version

### Post-1.0 Versioning

- Breaking changes increment MAJOR version
- New features increment MINOR version
- Bug fixes increment PATCH version

### Release Checklist

1. Update CHANGELOG.md with version and date
2. Update version in Cargo.toml
3. Run full test suite
4. Create git tag: `git tag -a v0.23.0 -m "Release v0.23.0"`
5. Push tag: `git push origin v0.23.0`
6. CI will automatically publish to crates.io

## CI/CD Pipeline

The project uses GitHub Actions for continuous integration:

1. **Conventional Commit Validation**: Ensures commit messages follow the standard
2. **Code Formatting**: Checks `cargo fmt`
3. **Linting**: Runs `cargo clippy` with warnings as errors
4. **Security Audit**: Checks for known vulnerabilities
5. **Tests**: Runs full test suite

All checks must pass before merging.

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue with reproduction steps
- **Features**: Open a GitHub Issue with use case description
- **Security**: See SECURITY.md for vulnerability reporting

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Assume good intentions
- Help create a welcoming environment

## License

By contributing to odos-sdk, you agree that your contributions will be licensed under the same license as the project (see LICENSE file).

## Recognition

Contributors will be recognized in the CHANGELOG.md and project README.

Thank you for contributing to odos-sdk!
