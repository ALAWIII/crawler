use super::CResult;
use num_cpus::get as get_logical;
use once_cell::sync::Lazy;
use reqwest::Url;
use std::sync::Arc;

pub static CPU_NUMBER: Lazy<usize> = Lazy::new(get_logical);
pub fn valid_url_format(url: &str) -> CResult<Url> {
    let parsed = Url::parse(url)?;
    Ok(parsed)
}

pub struct UrlData(pub Arc<Url>, pub String);

impl UrlData {
    pub fn get_url(&self) -> Arc<Url> {
        self.0.clone()
    }
    pub fn get_raw_page(&mut self) -> &str {
        &self.1
    }
}

#[derive(Debug)]
/// This structure for containing the
pub struct UrlParsedData(pub Arc<Url>, pub Vec<String>);
impl UrlParsedData {
    pub fn get_url(&self) -> Arc<Url> {
        self.0.clone()
    }
    pub fn get_parsed_text(&mut self) -> &mut Vec<String> {
        &mut self.1
    }
}
