# crawler [![License](https://img.shields.io/crates/l/crawler)](./LICENSE) [![crawler on crates.io](https://img.shields.io/crates/v/crawler)](https://crates.io/crates/crawler) [![crawler on docs.rs](https://docs.rs/crawler/badge.svg)](https://docs.rs/crawler) [![Source Code Repository](https://img.shields.io/badge/Code-On%20GitHub-blue?logo=GitHub)](https://github.com/ALAWIII/crawler)

## Web Crawler and Query Tool

This project is a command-line application that allows you to crawl web pages starting from a given URL,
retrieve documents, and search for specific text queries within those documents.

### Features

* Crawl and fetch documents from a specified starting URL.
* Search for a query text within the crawled documents.
* Configure the maximum number of documents to retrieve.
* Set a timeout for the crawling and query process.
* only support english language for tokenization !

### Usage

#### Command-Line Options

|Option|Description|Default Value|
|------|-----------|-------------|
|`-s, --start-point <URL>`|Provide a valid URL to start fetching and crawling.|N/A|
|`-q, --query <QUERY TEXT>`|Text query to search within the documents and retrieve the related ones.|N/A|
|`-m, --max-doc <NUMBER>`|Maximum number of documents to retrieve.|10|
|`-t, --timeout <SECONDS>`|Specifies the timeout (in seconds) for processing the request.|10|

#### Example

To run the program, use the following command:

```bash
./crawler -s https://example.com -q "sample query" -m 15 -t 20
```

This will:

1. Start crawling from `https://example.com`.
2. Search for `"sample query"` within the crawled documents.
3. Retrieve up to 15 documents.
4. Timeout after 20 seconds if the process takes too long.

### Installation

download binary directly from the [Release Page][__link0]!

Or compile it by yourself :

1. Clone the repository:
   ```bash
   git clone <repository-url>
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

### Requirements

* If you want to compile it by yourself (ensure Rust is installed on your machine).

### License

This project is licensed under the MIT License. See `LICENSE` for more details.

### Contributing

Contributions are welcome! Feel free to submit a pull request or open an issue.

---

## BLUEPRINT

built with performance in mind:

1. Tokio threadpools with channels between them to support I/O bound tasks.
1. Rayon for CPU bound tasks.
1. Database : Surrealdb.

All those to support scalability , maintainability and isolation of the main process !

![crawler blueprint](./blueprint.svg)

Happy crawling!


 [__link0]: https://github.com/ALAWIII/crawler/releases
