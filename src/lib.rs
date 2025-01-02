//! # Web Crawler and Query Tool
//!
//! This project is a command-line application that allows you to crawl web pages starting from a given URL,
//! retrieve documents, and search for specific text queries within those documents.
//!
//! ## Features
//!
//! - Crawl and fetch documents from a specified starting URL.
//! - Search for a query text within the crawled documents.
//! - Configure the maximum number of documents to retrieve.
//! - Set a timeout for the crawling and query process.
//!
//! ## Usage
//!
//! ### Command-Line Options
//!
//! | Option                      | Description                                                                                       | Default Value |
//! |-----------------------------|---------------------------------------------------------------------------------------------------|---------------|
//! | `-s, --start-point <URL>`   | Provide a valid URL to start fetching and crawling.                                               | N/A           |
//! | `-q, --query <QUERY TEXT>`  | Text query to search within the documents and retrieve the related ones.                          | N/A           |
//! | `-m, --max-doc <NUMBER>`    | Maximum number of documents to retrieve.                                                         | 10            |
//! | `-t, --timeout <SECONDS>`   | Specifies the timeout (in seconds) for processing the request.                                    | 10            |
//!
//! ### Example
//!
//! To run the program, use the following command:
//!
//! ```bash
//! ./crawler -s https://example.com -q "sample query" -m 15 -t 20
//! ```
//!
//! This will:
//! 1. Start crawling from `https://example.com`.
//! 2. Search for `"sample query"` within the crawled documents.
//! 3. Retrieve up to 15 documents.
//! 4. Timeout after 20 seconds if the process takes too long.
//!
//! ## Installation
//! download binary directly from the [Release Page](https://github.com/ALAWIII/crawler/releases)!
//!
//! Or compile it by yourself :
//!
//! 1. Clone the repository:
//!    ```bash
//!    git clone <repository-url>
//!    ```
//! 2. Build the project:
//!    ```bash
//!    cargo build --release
//!    ```
//!
//! ## Requirements
//!
//! - If you want to compile it by yourself (ensure Rust is installed on your machine).
//!
//! ## License
//!
//! This project is licensed under the MIT License. See `LICENSE` for more details.
//!
//! ## Contributing
//!
//! Contributions are welcome! Feel free to submit a pull request or open an issue.
//!
//! ---
//!
//! Happy crawling!
mod interface;
pub use interface::{get_args, run, CResult, Config};
mod database;
pub use database::*;
mod page_utils;
pub use page_utils::*;
mod go_spider;
pub use go_spider::start_process;
mod log_creation;
pub use log_creation::*;
