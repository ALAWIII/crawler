use crate::start_process;
use clap::{command, Arg};
use std::error::Error;

const HELP: &str = r#"
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading}
    {usage}

{all-args}{after-help}
"#;

pub type CResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pub start_point: String, // starting point for crawling
    pub query: String,       // user provided query to search in the documents!
    pub max_doc: usize,      // maximum number of documents the user wants to retrieve
    pub timeout: usize,
}
impl Config {
    pub fn get_url(&self) -> &str {
        &self.start_point
    }
    pub fn get_query(&self) -> &str {
        &self.query
    }
    pub fn get_max_doc(&self) -> usize {
        self.max_doc
    }
    pub fn get_timeout(&self) -> usize {
        self.timeout
    }
}
pub fn get_args() -> CResult<Config> {
    let matches = command!()
        .author("allawiii <alighraibeh87@gmail.com>")
        .about("Web Crawler").help_template(HELP)
        .next_line_help(true)
        .arg(
            Arg::new("start_point").short('s')
                .long("start-point")
                .value_name("URL link")
                .required(true)
                .num_args(1)
                .long_help("you must provide a valid url that will be used in order to start fetching and crawling!"),
        ).arg(
            Arg::new("query")
                .short('q')
                .long("query")
                .required(true)
                .num_args(1)
                .value_name("QUERY TEXT")
                .long_help("text query to search within the documents and retrive the related once"))
        .arg(
            Arg::new("max_docs")
            .short('m')
            .long("max-doc")
            .num_args(1)
            .required(false)
            .value_name("NUMBER")
            .long_help("maximum number of docs the user wants to retrive , if the number of crawled docs is less than specified then it will retrive them.")
            .default_value("10")
        ).arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .required(false)
                .num_args(1)
                .value_name("SECONDS")
                .default_value("10")
                .long_help("Specifies the period of time the program will spend processing the request before it stops. ")
        )
        .get_matches();
    let max_doc = matches
        .get_one::<String>("max_docs")
        .unwrap()
        .parse::<usize>()
        .expect("you have to provide valid maximum number!");
    let timeout = matches
        .get_one::<String>("timeout")
        .unwrap()
        .parse::<usize>()
        .expect("please specify a valid number of seconds");
    Ok(Config {
        start_point: matches.get_one::<String>("start_point").unwrap().into(),
        query: matches.get_one::<String>("query").unwrap().into(),
        max_doc,
        timeout,
    })
}

pub async fn run(config: Config) -> CResult<()> {
    start_process(config).await?;
    Ok(())
}
