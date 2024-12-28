use crate::{
    get_db_connection, get_log_failure, get_log_success, CResult, Document, DocumentId, Invindex,
    InvindexId, UrlParsedData, CPU_NUMBER,
};
use dashmap::DashMap;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::{self, iter::ParallelIterator};
use reqwest::Url;
use std::io::Write;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::watch;
use tokio::sync::{mpsc::Receiver, Semaphore};
use tokio::task::JoinHandle;
type TableTerms = DashMap<Arc<String>, Invindex>;

pub async fn analyze_pages(
    mut rcv3: Receiver<UrlParsedData>,
    mut shut_analyze: watch::Receiver<bool>,
) -> CResult<()> {
    let semaphore = Arc::new(Semaphore::new(*CPU_NUMBER));

    loop {
        tokio::select! {
            Some(UrlParsedData(url, page_text)) =rcv3.recv()=>{
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                tokio::spawn(async move {
                    let url_as_string = url.as_str().to_string();
                    let combined_tokens = combine_tokens(page_text, url)
                        .await
                        .expect("Failed to combine objects");

                    send_to_database(combined_tokens, url_as_string).await;
                    drop(permit);
                });
            },
            _ = shut_analyze.changed()=>{
                if *shut_analyze.borrow(){
                    let mut log = get_log_success().lock_owned().await;
                    writeln!(log ,"shutting down the analysis process")
                        .expect("failed to write to log file");
                    return Ok(());
                }
            }
        }
    }
}

/// splits the texts and clean it into single atomic tokens (words);
fn clean_and_tokenize(text: &str, url: Arc<Url>) -> Vec<Invindex> {
    text.split(|c: char| !c.is_alphabetic())
        .filter(|token| token.len() > 1)
        .map(move |token| {
            Invindex::new(InvindexId {
                term: Arc::new(token.to_owned()),
                doc_url: url.to_string(),
            })
        })
        .collect()
}

/// collects and stores all terms with their locations in Invindex object
fn combine_tokens(
    mut page_text: Vec<String>,
    url: Arc<Url>,
) -> JoinHandle<DashMap<Arc<String>, Invindex>> {
    tokio::task::spawn_blocking(move || {
        let term_locations = DashMap::new();
        let doc_length = AtomicUsize::new(0);
        let flatted_text = page_text.par_iter_mut().flat_map_iter(|text| {
            let tokens = clean_and_tokenize(text, url.clone());
            doc_length.fetch_add(tokens.len(), Ordering::SeqCst);
            tokens
        });
        let index = Arc::new(AtomicUsize::new(0));
        flatted_text.for_each(|mut token| {
            let i = index.load(Ordering::SeqCst);
            token.doc_length(doc_length.load(Ordering::SeqCst));
            term_locations
                .entry(token.get_term())
                .or_insert(token) //checks if the term exist otherwise inserting it and its current location
                .add_location(i);

            index.fetch_add(1, Ordering::SeqCst);
        });
        term_locations
    })
}

/// migrates term and urls to the database into correct tables ,
async fn send_to_database(combined_tokens: TableTerms, url_as_string: String) {
    let db = get_db_connection().await;
    let mut log = get_log_failure().lock_owned().await;
    for (_, v) in combined_tokens.into_iter() {
        let _: Option<()> = db
            .create("inv_index")
            .content(v)
            .await
            .unwrap_or_else(|err| {
                writeln!(log, "failed to create a record : {:?}", err)
                    .expect("failed to write to a log file");
                Some(())
            });
    }
    // when finishes it sends the url as string to the document table.
    let _: Option<()> = db
        .create("document")
        .content(Document {
            id: DocumentId { url: url_as_string },
        })
        .await
        .unwrap_or_else(|err| {
            writeln!(log, "failed to create a record : {:?}", err)
                .expect("failed to write to a log file");
            Some(())
        });
}
