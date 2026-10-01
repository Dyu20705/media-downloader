# openDownloader

openDownloader is a desktop application for inspecting a media URL, reviewing an acquisition plan, and downloading a single video, audio track, clip/chapter, thumbnail, or one subtitle language per job. It uses Tauri, Rust, React, and separately managed media tools.

## Support and limitations

The production package workflow targets 64-bit Windows, Linux, and Intel macOS. Managed media-tool availability is narrower than the desktop framework's possible platforms; consult [tool management](docs/tool-management.md) before relying on managed installs. Playlist URLs select one item by default; downloading a whole playlist is not supported. Custom output profiles and metadata-patch requests are unsupported. Media availability and formats depend on the source service and its terms.

Downloads run through backend processes without shell interpolation. Media URL details are removed from diagnostics and persisted history is sanitized; history is stored locally in SQLite and is not encrypted. Managed tool downloads are checked against pinned SHA-256 values before installation. See [security](docs/security.md) and [privacy](PRIVACY.md).

## Build from source

Prerequisites: Node.js 24, npm, Rust 1.96, and the [Tauri platform prerequisites](https://tauri.app/start/prerequisites/).

```sh
npm ci
npm run tauri dev
npm run tauri build
```

See [packaging](docs/packaging.md) for supported bundles and release status, and [troubleshooting](docs/troubleshooting.md) for user help.

## Maintained documentation

- [Architecture](docs/architecture.md)
- [Tool management](docs/tool-management.md)
- [Security](docs/security.md)
- [Privacy](PRIVACY.md)
- [Performance](docs/performance.md)
- [Packaging and releases](docs/packaging.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Changelog](CHANGELOG.md)
- [Third-party notices](THIRD_PARTY_NOTICES.md)
- [Security reporting](SECURITY.md)

## License

The application is licensed under the [MIT License](LICENSE). Third-party tool licenses are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
