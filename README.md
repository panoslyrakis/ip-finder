# IP Finder

A small desktop application built with Dioxus that looks up the IPv4 and IPv6 addresses of a given domain, and displays your device's current public IP address.

## What it does

- Shows your device's public IP address on launch
- Accepts a domain name or full URL as input
- Resolves and displays both the IPv4 and IPv6 addresses for that domain
- Handles input with or without protocol prefix (e.g. `wpmudev.com` or `https://wpmudev.com`)

## Requirements

- Rust (install via https://rustup.rs)
- macOS with Xcode Command Line Tools installed

## Getting started

Clone or download the project, then from the project root run:

```bash
cargo run
```

On the first run, Cargo will download and compile all dependencies. This may take a few minutes. Subsequent runs will be faster.

## Project structure

```
ip-finder/
  src/
    main.rs     Application code
  Cargo.toml    Dependencies and project metadata
  README.md     This file
```

## Dependencies

- `dioxus` with the `desktop` feature - UI framework
- `reqwest` - HTTP client used to fetch the device's public IP
- `tokio` - Async runtime

## Notes

- The device IP is fetched from `ifconfig.me` on startup and requires an internet connection
- DNS resolution is handled by the operating system, the same way a browser resolves domains
- Direct IP access in a browser will typically return a 403 response from most hosting providers, which is expected and normal
- The application window title can be changed in the `main` function via `with_title()`