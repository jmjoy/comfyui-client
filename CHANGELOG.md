# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/jmjoy/comfyui-client/compare/v0.3.0...v0.4.0) - 2025-05-01

### Added

- Derive Clone and add gifs in history ([#25](https://github.com/jmjoy/comfyui-client/pull/25))

### Fixed

- handle execution error and interruption events in main loop ([#26](https://github.com/jmjoy/comfyui-client/pull/26))

## [0.3.0](https://github.com/jmjoy/comfyui-client/compare/v0.2.0...v0.3.0) - 2025-04-19

### Other

- update event handling to use ComfyEvent and ConnectionEvent ([#23](https://github.com/jmjoy/comfyui-client/pull/23))
- remove JoinHandle from ClientBuilder and related functions ([#22](https://github.com/jmjoy/comfyui-client/pull/22))
- update ClientBuilder to return a JoinHandle for background task management ([#21](https://github.com/jmjoy/comfyui-client/pull/21))
- update README with API reference and WebSocket connection details ([#20](https://github.com/jmjoy/comfyui-client/pull/20))
- enhance logging setup in test environment ([#19](https://github.com/jmjoy/comfyui-client/pull/19))
- enhance event handling and output structure in meta and integration tests ([#18](https://github.com/jmjoy/comfyui-client/pull/18))
- update ClientBuilder to accept generic URL type and improve CI configuration ([#17](https://github.com/jmjoy/comfyui-client/pull/17))

## [0.2.0](https://github.com/jmjoy/comfyui-client/compare/v0.1.2...v0.2.0) - 2025-04-16

### Other

- Rename reconnect and add channel_bound method ([#16](https://github.com/jmjoy/comfyui-client/pull/16))
- crate features and minor cleanups ([#15](https://github.com/jmjoy/comfyui-client/pull/15))
- Enhance WebSocket error handling and reconnection logic in ClientBuilder ([#14](https://github.com/jmjoy/comfyui-client/pull/14))
- Refactor event handling in ComfyUI client ([#13](https://github.com/jmjoy/comfyui-client/pull/13))
- Add reconnect support to ClientBuilder and enhance event handling ([#12](https://github.com/jmjoy/comfyui-client/pull/12))
- Reduce post_prompt api ([#8](https://github.com/jmjoy/comfyui-client/pull/8))
- Update CI

## [0.1.2](https://github.com/jmjoy/comfyui-client/compare/v0.1.1...v0.1.2) - 2025-03-17

### Other

- Fix CI ([#6](https://github.com/jmjoy/comfyui-client/pull/6))
- Use release plz in CI ([#5](https://github.com/jmjoy/comfyui-client/pull/5))
- Adjust codes ([#4](https://github.com/jmjoy/comfyui-client/pull/4))
