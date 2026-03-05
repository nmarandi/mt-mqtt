# Contributing to MT-MQTT

First off, thank you for considering contributing to MT-MQTT! 🎉

## Code of Conduct

This project and everyone participating in it is expected to be respectful and professional. Please be kind and courteous in all interactions.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When you create a bug report, include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples** (code snippets, configuration, etc.)
- **Describe the behavior you observed** and what you expected
- **Include version information** (Rust version, OS, MT-MQTT version)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion:

- **Use a clear and descriptive title**
- **Provide a detailed description** of the suggested enhancement
- **Explain why this enhancement would be useful**
- **List any similar features** in other MQTT brokers if applicable

### Pull Requests

1. Fork the repository
2. Create a new branch from `main` (`git checkout -b feature/your-feature-name`)
3. Make your changes following the coding standards below
4. Add or update tests as appropriate
5. Update documentation if needed
6. Ensure all tests pass (`cargo test`)
7. Ensure code is properly formatted (`cargo fmt`)
8. Ensure there are no clippy warnings (`cargo clippy`)
9. Commit your changes with clear, descriptive commit messages
10. Push to your fork and submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Building

```bash
# Clone the repository
git clone https://github.com/nmarandi/mt-mqtt.git
cd mt-mqtt

# Build the project
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Running the Broker

```bash
# Run in development mode
cargo run

# Run in release mode
cargo run --release

# With SQLite persistence
cargo run --features sqlite
```

## Coding Standards

### Rust Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html)
- Use `rustfmt` for consistent formatting: `cargo fmt`
- Address all `clippy` warnings: `cargo clippy`
- Write idiomatic Rust code

### Code Organization

- Keep modules focused and cohesive
- Use meaningful names for functions, variables, and types
- Add doc comments for public APIs
- Keep functions reasonably sized and focused

### Testing

- Write tests for new features
- Maintain or improve test coverage
- Include both unit and integration tests where appropriate
- Test edge cases and error conditions

### Documentation

- Update README.md for user-facing changes
- Add doc comments to public APIs
- Update relevant documentation in the `docs/` folder
- Include examples for new features

### Commit Messages

- Use clear, descriptive commit messages
- Start with a verb in the imperative mood (e.g., "Add", "Fix", "Update")
- Keep the first line under 72 characters
- Add a detailed description if necessary

Example:
```
Add support for MQTT 5.0 reason codes

- Implement reason code parsing in CONNACK packets
- Add tests for various reason codes
- Update documentation with MQTT 5.0 notes
```

## Project Structure

```
mt-mqtt/
├── src/
│   ├── auth.rs           # Authentication and ACL
│   ├── broker/           # Core broker logic
│   ├── client.rs         # Client connection handling
│   ├── protocol/         # MQTT protocol implementation
│   ├── session.rs        # Session management
│   ├── topic.rs          # Topic tree and routing
│   └── persistence/      # Persistence backends
├── tests/                # Integration tests
├── benches/              # Performance benchmarks
├── docs/                 # Documentation
└── examples/             # Example code

```

## Testing Guidelines

### Unit Tests

- Place unit tests in the same file as the code they test
- Use the `#[cfg(test)]` attribute
- Test individual functions and methods in isolation

### Integration Tests

- Place integration tests in the `tests/` directory
- Test complete workflows and scenarios
- Use realistic MQTT client interactions

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

## Performance Considerations

- Be mindful of allocations in hot paths
- Use async/await appropriately
- Benchmark performance-critical changes
- Run benchmarks before and after changes: `cargo bench`

## Documentation

- Keep README.md up to date
- Add examples for new features
- Update relevant documentation in `docs/`
- Use doc comments liberally (`///` for public items)

## MQTT Protocol Compliance

- Follow the [MQTT 3.1.1 specification](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html)
- Ensure backward compatibility
- Test with standard MQTT clients (mosquitto, paho, etc.)

## Security

- Never commit sensitive information (passwords, keys, etc.)
- Report security vulnerabilities privately (see SECURITY.md if it exists)
- Follow secure coding practices
- Use bcrypt for password hashing (already implemented)

## Questions?

Feel free to open an issue with your question, or reach out to the maintainers.

## License

By contributing to MT-MQTT, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to MT-MQTT! 🚀
