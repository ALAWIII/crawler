use super::{
    fetch_pages, get_db_connection, page_utils::analyze_pages, parse_pages, qurey_database,
    valid_url_format, CResult, Config, TermDocRecord, UrlData, UrlParsedData, CPU_NUMBER,
};
use std::time::Duration;
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        watch,
    },
    task::JoinHandle,
};
use url::Url;
pub async fn start_process(config: Config) -> CResult<()> {
    let _ = get_db_connection().await;
    let (snd1, rcv1) = channel::<UrlData>(*CPU_NUMBER); // for pool A
    let (snd2, rcv2) = channel(*CPU_NUMBER); // for pool B
    let (snd3, rcv3) = channel(*CPU_NUMBER); // for pool C
    let (shut_send, shut_recv) = watch::channel(false);
    let initial_url = valid_url_format(config.get_url())?;
    // starts to fetch and download the pages
    let shut_fetch = shut_recv.clone();
    let shut_parse = shut_recv.clone();
    let shut_analyze = shut_recv.clone();
    //---------------------- starting all the processes ------------------------------
    let fetching_pages = start_fetch_pages(snd1, rcv2, shut_fetch);
    snd2.send(initial_url).await?; // critical for sending the inital crawling url , must be awaited to garuntee it will start with it.

    // for transforming the recived pages into the correct format
    let parsing_pages = start_parse_pages(snd2, snd3, rcv1, shut_parse);
    let analyzing_pages = start_analyze_pages(rcv3, shut_analyze);

    let stop = stop_program(shut_send, config.timeout as u64);

    //------------------------ awaiting all the process untill shutdown ---------------
    tokio::join!(fetching_pages, parsing_pages, analyzing_pages, stop); // this will be joined with the other major tasks

    //---------------------- finishing up and printing the top related documents-------
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

fn start_fetch_pages(
    snd1: Sender<UrlData>,
    rcv2: Receiver<Url>,
    shut_fetch: watch::Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = fetch_pages(snd1, rcv2, shut_fetch).await {
            println!("Fetching pages task error: {:?}", e);
            //eprintln!("Fetching pages task error: {:?}", e)
        };
    })
}
fn start_parse_pages(
    snd2: Sender<Url>,
    snd3: Sender<UrlParsedData>,
    rcv1: Receiver<UrlData>,
    shut_parse: watch::Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = parse_pages(snd2, snd3, rcv1, shut_parse).await {
            println!("Parsing pages task error: {:?}", e);
        }
    })
}

fn start_analyze_pages(
    rcv3: Receiver<UrlParsedData>,
    shut_analyze: watch::Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async {
        if let Err(e) = analyze_pages(rcv3, shut_analyze).await {
            println!("Analyzing pages task error: {:?}", e);
        };
    })
}

fn stop_program(shut_send: watch::Sender<bool>, timeout: u64) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(timeout)).await;
        let _ = shut_send.send(true);
    })
}
