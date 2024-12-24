use super::{fetch_pages, parse_pages, valid_url_format, UrlData, CPU_NUMBER};
use crate::get_db_connection;
use crate::{page_utils::analyze_pages, CResult, Config};
use std::time::Duration;
use tokio::sync::mpsc::channel;
//use url::Url;

pub async fn start_process(config: Config) -> CResult<()> {
    let _ = get_db_connection().await;
    let (snd1, rcv1) = channel::<UrlData>(*CPU_NUMBER); // for pool A
    let (snd2, rcv2) = channel(*CPU_NUMBER); // for pool B
    let (snd3, rcv3) = channel(*CPU_NUMBER); // for pool C
    let initial_url = valid_url_format(config.get_url())?;

    let pag = tokio::spawn(async move {
        fetch_pages(snd1, rcv2)
            .await
            .expect("stopping get pages process");
    });
    snd2.send(initial_url).await?; // critical for sending the inital crawling url , must be awaited to garuntee it will start with it
    let cloned_snd2 = snd2.clone();
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
    Ok(())
}
