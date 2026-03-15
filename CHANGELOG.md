# Changelog

All notable changes to the MT-MQTT broker will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Secure password hashing with bcrypt (cost factor 12)
- Topic-based Access Control Lists (ACL) with wildcard support
- Will Messages (Last Will and Testament) implementation
- `add_user_with_hash()` method for loading pre-hashed passwords
- Comprehensive documentation for authentication and will messages
- Review report document

### Changed
- Authentication module now stores bcrypt hashes instead of plaintext passwords
- Password verification uses constant-time comparison
- Updated documentation to reflect secure implementation
- Improved .gitignore to exclude common development artifacts

### Fixed
- MQTT 3.1.1 will message decoder now correctly handles protocol version
- Wildcard pattern matching in ACL now correctly handles `#` for zero or more levels
- Broker now clears existing will message on client reconnect
- Code formatting issues in decoder

### Security
- Replaced cleartext password storage with bcrypt hashing
- Implemented constant-time password comparison to prevent timing attacks
- Added proper wildcard validation in ACL patterns

## [0.1.0] - 2026-02-18

### Added
- Core MQTT 3.1.1 protocol implementation
- QoS 0, 1, and 2 support with proper acknowledgment flows
- Wildcard subscriptions with `+` (single-level) and `#` (multi-level)
- Retained messages functionality
- Persistent sessions with state preservation across reconnects
- Optional SQLite backend for disk persistence (feature flag)
- Packet identifier management with automatic allocation
- Message state tracking for in-flight messages
- Topic tree with efficient trie-based matching
- Keep-alive support (PINGREQ/PINGRESP)
- Graceful disconnect handling
- Comprehensive test suite (48 unit + 13 integration tests)
- Benchmarking suite for performance testing
- Detailed documentation and examples

### Core Features
- TCP server listening on port 1883
- Asynchronous I/O built on Tokio
- Multi-subscriber support with efficient fanout
- Subscribe/Unsubscribe operations
- CONNECT/CONNACK handshake
- Clean session support

## [0.0.1] - Initial Development

### Added
- Initial project structure
- Basic MQTT packet parsing
- Topic tree implementation
- Protocol definitions

[Unreleased]: https://github.com/nmarandi/mt-mqtt/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/nmarandi/mt-mqtt/releases/tag/v0.1.0
