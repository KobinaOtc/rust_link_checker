# Overview

As a software engineer looking to deepen my understanding of systems programming and high-performance concurrency, I built this project to gain hands-on experience with Rust's asynchronous ecosystem, memory safety guarantees, and command-line tooling.

The **Concurrent Website Link Checker & Uptime Monitor** is a fast, lightweight CLI utility written in Rust. It operates in two modes: crawling a target URL to extract and validate all internal and external links on a page, or reading a local configuration file of endpoints to monitor their uptime. Using an asynchronous runtime and semaphore-based rate limiting, the application checks HTTP status codes concurrently, enforces request timeouts, sorts broken links to the top, displays a color-coded health table in the terminal, and optionally exports the full report to JSON or CSV.

My purpose in creating this software was to move beyond basic syntax and solve a practical web reliability problem while mastering core Rust concepts—specifically `async`/`await` task spawning, shared state across threads with `Arc` and `Semaphore`, HTML parsing, URL resolution, and structured error handling using `Result` and `Option`.

[Software Demo Video](http://youtube.link.goes.here)

# Development Environment

* **Editor & OS:** Visual Studio Code on macOS
* **Build System & Package Manager:** Cargo and Rustup
* **Version Control:** Git and GitHub

The software is written entirely in **Rust** and leverages the following open-source crates:
* **Tokio:** Asynchronous runtime for spawning concurrent network tasks and enforcing concurrency limits via `Semaphore`.
* **Reqwest:** Ergonomic, asynchronous HTTP client configured with TLS support and request timeouts.
* **Scraper:** HTML parsing and CSS selector engine used to extract `<a href>` anchor tags.
* **Url:** Standards-compliant URL parsing and relative path resolution.
* **Clap:** Derive-based command-line argument parser for handling CLI flags (`--url`, `--file`, `--concurrency`, `--output`).
* **Comfy-Table:** Renders formatted, color-coded UTF-8 tables directly in the terminal.
* **Serde, Serde JSON, & CSV:** Serialization libraries used to export health reports to `.json` and `.csv` formats.

# Useful Websites

* [The Rust Programming Language Book](https://doc.rust-lang.org/book/)
* [Tokio Asynchronous Rust Tutorial](https://tokio.rs/tokio/tutorial)
* [Reqwest Crate Documentation](https://docs.rs/reqwest/latest/reqwest/)
* [Scraper Crate Documentation](https://docs.rs/scraper/latest/scraper/)
* [Clap Crate Documentation](https://docs.rs/clap/latest/clap/)
* [Serde Serialization Framework](https://serde.rs/)
* [Comfy-Table Crate Documentation](https://docs.rs/comfy-table/latest/comfy_table/)