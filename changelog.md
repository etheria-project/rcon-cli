# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-09-08

Initial Release ! 🎉

### Added
- Initial release of RCON CLI tool
- Full RCON protocol support for Minecraft servers
- Multiple operation modes: single command, interactive, ping, info, players
- Async/await implementation with Tokio
- Rich output formatting with colored text and JSON support
- Auto-reconnection capabilities
- Response fragmentation handling
- Comprehensive error handling and validation
- Configurable logging levels
- Library API for programmatic access
- Support for environment variables (RCON_PASSWORD)
- Cross-platform compatibility

### Features
- Command execution with timing information
- Interactive mode with command history
- Server connectivity testing
- Detailed server information retrieval
- Player listing with optional UUID support
- Flexible connection configuration
- Multiple output formats (text/JSON)

---

## Release Template

When creating a new release, copy this template:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features

### Changed
- Changes to existing functionality

### Deprecated
- Soon-to-be removed features

### Removed
- Removed features

### Fixed
- Bug fixes

### Security
- Security improvements
```
