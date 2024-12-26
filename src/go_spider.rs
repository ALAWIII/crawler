use crate::TermDocRecord;

use super::{
    fetch_pages, get_db_connection, page_utils::analyze_pages, parse_pages, qurey_database,
    valid_url_format, CResult, Config, UrlData, CPU_NUMBER,
};

use std::time::Duration;
use tokio::sync::mpsc::channel;

pub async fn start_process(config: Config) -> CResult<()> {
    let _ = get_db_connection().await;
    let (snd1, rcv1) = channel::<UrlData>(*CPU_NUMBER); // for pool A
    let (snd2, rcv2) = channel(*CPU_NUMBER); // for pool B
    let (snd3, rcv3) = channel(*CPU_NUMBER); // for pool C
    let initial_url = valid_url_format(config.get_url())?;
    // starts to fetch and download the pages
    let pag = tokio::spawn(async move {
        fetch_pages(snd1, rcv2)
            .await
            .expect("stopping get pages process");
    });
    snd2.send(initial_url).await?; // critical for sending the inital crawling url , must be awaited to garuntee it will start with it
    let cloned_snd2 = snd2.clone();
    // for transforming the recived pages into the correct format
    let pages_parser = tokio::spawn(async {
        parse_pages(cloned_snd2, snd3, rcv1)
            .await
            .expect("stopping parsing pages");
    });
    let page_analyzing = tokio::spawn(async {
        analyze_pages(rcv3).await.expect("stopping page analyzing");
    });

    let timeout = config.timeout;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(timeout as u64)).await;
        drop(snd2);
    });
    tokio::join!(pag, pages_parser, page_analyzing); // this will be joined with the other major tasks
    let list_docs = qurey_database(&config.query).await;
    print_docs(list_docs, config.max_doc);

    Ok(())
}

fn print_docs(mut docs: Vec<TermDocRecord>, number_docs_needed: usize) {
    let mut i = 0;
    while !docs.is_empty() && i < number_docs_needed {
        let doc = docs.pop().unwrap();
        println!("{}", doc.get_url());
        i += 1
    }
}
