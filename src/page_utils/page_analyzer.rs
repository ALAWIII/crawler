use super::{UrlParsedData, CPU_NUMBER};
use crate::get_db_connection;
use crate::CResult;
use crate::ItemId;
use crate::{Document, DocumentId, Invindex, InvindexId};
use dashmap::DashMap;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::{self, iter::ParallelIterator};
use reqwest::Url;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc::Receiver, Semaphore};

pub async fn analyze_pages(mut rcv3: Receiver<UrlParsedData>) -> CResult<()> {
    let semaphore = Arc::new(Semaphore::new(*CPU_NUMBER));

    while let Some(UrlParsedData(url, mut page_text)) = rcv3.recv().await {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        tokio::spawn(async move {
            let urll = url.as_str().to_string();
            let combined_tokens = tokio::task::spawn_blocking(move || {
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
                        .or_insert(token)
                        .add_location(i);

                    index.fetch_add(1, Ordering::SeqCst);
                });
                term_locations
            })
            .await
            .expect("Blocking task failed");
            let db = get_db_connection().await;
            for (_, v) in combined_tokens.into_iter() {
                let _: surrealdb::Result<Option<ItemId>> = db.create("inv_index").content(v).await;
            }

            let _: surrealdb::Result<Option<ItemId>> = db
                .create("document")
                .content(Document {
                    id: DocumentId { url: urll },
                })
                .await;

            drop(permit);
        });
    }
    Ok(())
}

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
