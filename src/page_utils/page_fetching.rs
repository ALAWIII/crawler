use super::UrlData;
use super::{CResult, CPU_NUMBER};
use reqwest::{Client, Url};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Semaphore;
use tokio::time::Duration;
pub async fn fetch_pages(snd1: Sender<UrlData>, mut rcv2: Receiver<Url>) -> CResult<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .pool_max_idle_per_host(10) // max 10 connections per host that used to keep past connections for future reuse.
        .build()
        .unwrap();

    let semaphore = Arc::new(Semaphore::new(*CPU_NUMBER));
    let mut visited_docs: HashSet<Arc<Url>> = HashSet::new();

    while let Some(url) = rcv2.recv().await {
        let cloned_urls = Arc::new(url);
        // println!("{}", cloned_urls);
        if !visited_docs.contains(&cloned_urls.clone()) {
            visited_docs.insert(cloned_urls.clone());
            let cloned_snd1 = snd1.clone();

            let client_clone = client.clone(); // not a deep clone!
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            tokio::spawn(async move {
                if let Ok(response) = client_clone.get(cloned_urls.as_str()).send().await {
                    cloned_snd1
                        .send(UrlData(cloned_urls.clone(), response.text().await.unwrap()))
                        .await
                        .expect("failed to send the url and its data");
                    //println!("{}", cloned_urls)
                    drop(permit);
                };
            });
        };
    }

    Ok(())
}
