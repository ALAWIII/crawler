use super::{CResult, UrlData, CPU_NUMBER};
//use crate::get_log_success;
use reqwest::{Client, Url};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::Duration;
pub async fn fetch_pages(
    snd1: Sender<UrlData>,
    mut rcv2: Receiver<Url>,
    mut shut_fetch: watch::Receiver<bool>,
) -> CResult<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .pool_max_idle_per_host(10) // max 10 connections per host that used to keep past connections for future reuse.
        .build()
        .unwrap();

    let semaphore = Arc::new(Semaphore::new(*CPU_NUMBER));
    let mut visited_docs: HashSet<Arc<Url>> = HashSet::new();

    loop {
        tokio::select! {
            _= shut_fetch.changed()=>{
                if *shut_fetch.borrow(){
                    println!("shutting down the fetching channel");
                    return Ok(());
                }
            },
            Some(url) =rcv2.recv() =>{
                let cloned_url = Arc::new(url);
                // println!("{}", cloned_urls);
                if !visited_docs.contains(&cloned_url.clone()) {
                    visited_docs.insert(cloned_url.clone());
                    let cloned_snd1 = snd1.clone();

                    let client_clone = client.clone(); // not a deep clone!
                    let permit = semaphore.clone().acquire_owned().await.unwrap();
                    fetch_data_helper(client_clone, cloned_snd1, cloned_url, permit);
                };
            }

        }
    }
}

fn fetch_data_helper(
    client: Client,
    snd1: Sender<UrlData>,
    url: Arc<Url>,
    permit: OwnedSemaphorePermit,
) {
    tokio::spawn(async move {
        if let Ok(response) = client.get(url.as_str()).send().await {
            if let Ok(res) = response.text().await {
                snd1.send(UrlData(url.clone(), res)).await.unwrap_or(());
            }
            drop(permit);
        };
    });
}
