//! page parser for extracting the text and urls (pool B)
//! combining all the text then sending it to the analysis and storage pool via the communication channel
//! sending the url's back to the get_pages function via the channel
use super::{valid_url_format, UrlData, UrlParsedData, CPU_NUMBER};

use crate::{get_log_failure, get_log_success, CResult};
use once_cell::sync::Lazy;
use reqwest::Url;
use scraper::{node::Text, Html};
use std::collections::HashSet;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    {watch, Semaphore},
};

static FORBIDDEN: Lazy<HashSet<&str>> = Lazy::new(|| {
    HashSet::from([
        "script", "link", "meta", "base", "noscript", "template", "iframe", "object", "embed",
        "style",
    ])
});

/// Brings the raw html pages from the pool A and then parses them to extract the texts and links.
///
/// It sends the extracted links back to pool A,
/// while sending the extracted text as a vector with its corresponding url to pool 3;
pub async fn parse_pages(
    snd2: Sender<Url>,
    snd3: Sender<UrlParsedData>,
    mut rcv1: Receiver<UrlData>,
    mut shut_parse: watch::Receiver<bool>,
) -> CResult<()> {
    let semaphore = Arc::new(Semaphore::new(*CPU_NUMBER));
    loop {
        tokio::select! {

            Some(urldata) = rcv1.recv() =>{
                let mut log= get_log_failure().lock_owned().await;
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let cloned_snd2 = snd2.clone();
                let cloned_snd3 = snd3.clone();
                tokio::spawn(async move {
                    let url_parsed_data = text_filter(urldata, cloned_snd2);
                    cloned_snd3
                        .send(url_parsed_data)
                        .await
                        .map_err(|e| {
                            writeln!(log,"error sending to pool 3: {:?}", e)
                                .expect("failed to write to log file");
                        })
                        .ok(); // Consume the Result to satisfy type checks
                     drop(permit);
                });
            }
            _= shut_parse.changed()=>{
                if *shut_parse.borrow(){
                    let mut log = get_log_success().lock_owned().await;
                    writeln!{log,"shutting down the parsing channel"}
                        .expect("failed to write to log file");
                    break;
                }
            },
        };
    }

    Ok(())
}

/// checking whether a text is good by examining the percentage = alpha_count/total_length >=0.5
fn is_good_text(text: &Text) -> bool {
    let text_length = text.len() as f64;
    if text_length <= 2.0 {
        return false;
    }
    let alpha_count = text.chars().filter(|c| c.is_alphabetic()).count() as f64;
    let percentage = alpha_count / text_length;

    percentage >= 0.6
}

/// returns all embeded urls + all texts
pub fn text_filter(mut urldata: UrlData, snd2: Sender<Url>) -> UrlParsedData {
    let document = Html::parse_document(urldata.get_raw_page());
    let mut full_text = vec![];
    // if the current node is forbidden (e.g not a html element like: script) then it will not
    // try to add its text content because its css or javascript code!!
    let mut counter_skip_scussive_nodes = 0;
    document.tree.into_iter().for_each(|n| {
        if n.is_element() {
            let element = n.as_element().unwrap();

            if FORBIDDEN.contains(element.name()) {
                counter_skip_scussive_nodes = 3;
                return;
            }
            if element.name() == "a" {
                element.attr("href").map(|l| {
                    valid_url_format(l).map(|u| {
                        let cloned_snd2 = snd2.clone();
                        tokio::spawn(async move {
                            // potentinal skippin and performance drawback
                            if let Err(e) = cloned_snd2.send(u).await {
                                let mut log = get_log_failure().lock_owned().await;
                                writeln!(log, "sending url to pool A: {:?}", e)
                                    .expect("Failed to write to a log file");
                            };
                        });
                    })
                });
            }
        }
        if counter_skip_scussive_nodes != 0 {
            counter_skip_scussive_nodes -= 1;
            return;
        }

        if n.is_text() {
            let text = n.as_text().unwrap();

            if is_good_text(text) {
                full_text.push(text.to_lowercase());
            }
        }
    });
    UrlParsedData(urldata.get_url(), full_text)
}
