use crawler::{
    fetch_pages, get_db_connection, parse_pages, start_process, text_filter, unify_docs,
    valid_url_format, CResult, Config, Document, DocumentId, Invindex, InvindexId, ItemId,
    TermDocRecord, TermDocRecordId, UrlData, UrlParsedData,
};
use dashmap::DashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Url;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::env;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::sync::atomic::AtomicUsize;
use std::{
    collections::HashSet,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc::channel;
use tokio::sync::watch;
static URL: &str = "https://en.wikipedia.org/wiki/Alan_Turing";

// for getting the doc texts!
async fn get_pages_raw_text(link: &str) -> String {
    reqwest::get(link).await.unwrap().text().await.unwrap()
}

#[test]
fn url_to_string() {
    let url = "https://www.scrapingcourse.com/ecommerce/";
    let parsed = Url::parse(url);
    assert_eq!(parsed.unwrap().to_string(), url);
}

#[tokio::test]
async fn start_process_check() {
    let config = Config {
        start_point: URL.to_string(),
        query: "eat".to_string(),
        max_doc: 10,
        timeout: 2,
    };
    start_process(config).await;
    println!("done")
}

// testing fetch pages logic
#[tokio::test]
async fn fetch_pages_logic() {
    let (snd1, mut rcv1) = channel(10);
    let (snd2, mut rcv2) = channel(10);
    let url = Url::parse(URL).unwrap();
    let (shut_sx, shut_rx) = watch::channel(false);
    let shit = shut_rx.clone();
    let get_pages_task = tokio::spawn(async move {
        fetch_pages(snd1.clone(), rcv2, shit)
            .await
            .expect("failed continue excecuting")
    });
    snd2.send(url.clone()).await.expect("failed to sending msg");

    let response = rcv1.recv().await.expect("failed to receive msg");
    shut_sx.send(true);
    let value = *shut_rx.borrow();
    dbg!(value);
    get_pages_task.await.expect("get_pages task failed");
    //println!("{}", response.0);
    assert_eq!(url.as_str(), response.get_url().as_str());
}

//testing page_parsing logic
#[tokio::test]
async fn page_parsing_stop() {
    let (snd1, rcv1) = channel(12);
    let (snd2, rcv2) = channel::<Url>(12);
    let (snd3, mut rcv3) = channel::<UrlParsedData>(12);
    let (shut_snd, shut_rcv) = watch::channel(false);

    let txt = get_pages_raw_text(URL).await;
    let expected_url_parsed_data = text_filter(
        UrlData(Arc::new(Url::parse(&URL).unwrap()), txt.clone()),
        snd2.clone(),
    );
    let url_data = UrlData(Arc::new(Url::parse(&URL).unwrap()), txt);
    snd1.send(url_data.clone()).await;

    let parse_handle = tokio::spawn(async move {
        parse_pages(snd2, snd3, rcv1, shut_rcv).await;
    });

    let txto = rcv3.recv().await.unwrap().1;
    shut_snd.send(true);
    assert_eq!(txto, expected_url_parsed_data.1);
    parse_handle.await;
    assert!(*shut_snd.borrow());
    //dbg!(txto);
}
//---------------------------------------
#[tokio::test]
async fn parse_document1() {
    let response = reqwest::get(URL).await.unwrap().text().await.unwrap();
    let document = Html::parse_document(&response);
    let selector = Selector::parse("body *").unwrap();
    let raw_text = document
        .select(&selector)
        .filter_map(|element| element.text().next())
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", raw_text);
}
#[tokio::test]
async fn parse_document2() {
    let response = reqwest::get(URL).await.unwrap().text().await.unwrap();
    let document = Html::parse_document(&response);
    let selector = Selector::parse("p, div, span").unwrap();
    let raw_text = document
        .select(&selector)
        .flat_map(|element| element.text())
        .map(|text| text.trim().to_lowercase())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", raw_text);
}
#[tokio::test]
async fn parse_document3() {
    let response = reqwest::get(URL).await.unwrap().text().await.unwrap();
    let document = Html::parse_document(&response);
    let selector = Selector::parse("p, div, span, h1, h2, li").unwrap();
    let raw_text = document
        .select(&selector)
        .filter(|element| !["style", "script", "noscript"].contains(&element.value().name()))
        .flat_map(|element| element.text())
        .map(|text| text.trim().to_lowercase())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();
    println!("{:?}", raw_text);
}
// --------------------------------pool B tests---------------------
async fn extract_links() -> HashSet<Url> {
    let response = reqwest::get(URL).await.unwrap().text().await.unwrap();
    let document = Html::parse_document(&response);
    let mut urls = HashSet::new();
    document.tree.into_iter().for_each(|n| {
        if n.is_element() {
            let element = n.as_element().unwrap();
            if element.name() == "a" && element.attr("href").is_some() {
                let link = element.attr("href").unwrap();
                if let Ok(url) = valid_url_format(link) {
                    urls.insert(url);
                }
            }
        }
    });
    urls
}

/// tests whether the sended urls by the channel already contained as expected,
/// tests the number of sent urls as expected
/// , for testing the logic
async fn filter_text(func: fn(&Url)) -> (HashSet<Url>, UrlParsedData) {
    let (snd, mut rcv) = channel(10);
    let document = get_pages_raw_text(URL).await;
    let parsed_url = Arc::new(Url::parse(URL).unwrap());
    let urldata = UrlData(parsed_url, document);
    let url_parsed_data = tokio::spawn(async { text_filter(urldata, snd) });

    let mut output_urls = HashSet::new();
    while let Some(url) = rcv.recv().await {
        func(&url);
        output_urls.insert(url);
    }
    (output_urls, url_parsed_data.await.unwrap())
}

/// should pass the test , it test
#[tokio::test]
async fn filter_text_contains_all_urls() {
    let expected_urls = extract_links().await;
    let (output_urls, url_parsed_data) = filter_text(|url| println!("{}", url.as_str())).await;
    assert_eq!(output_urls, expected_urls);
    assert_eq!(output_urls.len(), expected_urls.len());
}
#[tokio::test]
#[should_panic]
async fn filter_text_not_contains_all_urls() {
    let expected_urls = extract_links().await;
    let (output_urls, url_parsed_data) = filter_text(|url| println!("{}", url.as_str())).await;
    assert_ne!(output_urls, expected_urls);
    assert_ne!(output_urls.len(), expected_urls.len());
}
#[tokio::test]
async fn filter_text_print_text() {
    let expected_urls = extract_links().await;
    let (output_urls, mut url_parsed_data) = filter_text(|url| {}).await;
    println!("{:?}", url_parsed_data.get_parsed_text());
}
//----------------------test backpressure-----------------

#[tokio::test]
async fn filter_text_heavy_channel_load() {
    let expected_urls = extract_links().await;
    let (snd, mut rcv) = channel(10);

    let document = get_pages_raw_text(URL).await;
    let parsed_url = Arc::new(Url::parse(URL).unwrap());
    let urldata = UrlData(parsed_url, document);
    let url_parsed_data = tokio::spawn(async { text_filter(urldata, snd) });
    //let mut output_urls = HashSet::new();
    let mut counter = 0;
    while let Some(url) = rcv.recv().await {
        //println!("{}", url.as_str());

        assert!(expected_urls.contains(&url));
        counter += 1;
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    //assert_eq!(expected_urls, output_urls);
    let result = url_parsed_data.await;
    assert!(result.is_ok());
    println!("{:?}", result.unwrap());
}
//-------------------------test cleaning texts----------------------

fn clean_and_tokenize(text: &str) -> Vec<String> {
    let mut big_tokens = vec![];
    let mut token = "".to_string();
    text.chars().for_each(|c| {
        if c.is_alphabetic() {
            token.push(c);
        } else {
            if token.len() > 1 {
                big_tokens.push(token.clone());
            }
            token.clear();
        }
    });
    if token.len() > 1 {
        big_tokens.push(token.clone());
    }
    big_tokens
}
fn clean_and_tokenize2(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphabetic())
        .filter_map(|token| {
            if token.len() > 1 {
                return Some(token.to_string());
            }
            None
        })
        .collect()
}
#[test]
fn clean_text1() {
    let expected1 = ["bank", "of", "england", "note"];
    let expected2 = ["in", "turing", "was", "prosecuted", "for"];
    let expected3 = ["for", "the", "appalling", "way", "turing", "was", "treated"];
    let expected4 = [
        "turing",
        "father",
        "was",
        "the",
        "son",
        "of",
        "clergyman",
        "the",
        "rev",
        "john",
        "robert",
        "turing",
        "from",
        "scottish",
        "family",
        "of",
        "merchants",
        "that",
    ];
    let output1 = clean_and_tokenize("bank of england £50 note");
    let output2 = clean_and_tokenize("in 1952, turing was prosecuted for ");
    let output3 = clean_and_tokenize(" for \"the appalling way [turing] was treated\"");
    let output4 = clean_and_tokenize(" turing's father was the son of a clergyman, the rev.\u{a0}john robert turing, from a scottish family of merchants that a  ");

    assert_eq!(output1, expected1);
    assert_eq!(output2, expected2);
    assert_eq!(output3, expected3);
    assert_eq!(output4, expected4);
}
#[test]
fn clean_text2() {
    let expected1 = ["bank", "of", "england", "note"];
    let expected2 = ["in", "turing", "was", "prosecuted", "for"];
    let expected3 = ["for", "the", "appalling", "way", "turing", "was", "treated"];
    let expected4 = [
        "turing",
        "father",
        "was",
        "the",
        "son",
        "of",
        "clergyman",
        "the",
        "rev",
        "john",
        "robert",
        "turing",
        "from",
        "scottish",
        "family",
        "of",
        "merchants",
        "that",
    ];
    let output1 = clean_and_tokenize2("bank of england £50 note");
    let output2 = clean_and_tokenize2("in 1952, turing was prosecuted for ");
    let output3 = clean_and_tokenize2(" for \"the appalling way [turing] was treated\"");
    let output4 = clean_and_tokenize2(" turing's father was the son of a clergyman, the rev.\u{a0}john robert turing, from a scottish family of merchants that a  ");

    assert_eq!(output1, expected1);
    assert_eq!(output2, expected2);
    assert_eq!(output3, expected3);
    assert_eq!(output4, expected4);
}
#[test]
fn parallel_text_sequintial_tokenization() {
    let texts = ["jump to content failed failed failed", "contact us"];
    let flatted_text: Vec<String> = texts
        .into_par_iter()
        .flat_map_iter(clean_and_tokenize2)
        .collect();
    let (index, value) = flatted_text
        .iter()
        .enumerate()
        .find(|(x, e)| **e == "jump")
        .unwrap();
    println!("{:?}", flatted_text);
    assert_eq!(flatted_text[index + 1], "to");
    assert_eq!(flatted_text[index + 2], "content");
    assert_eq!(flatted_text[index + 3], "failed");
    assert_eq!(flatted_text[index + 4], "failed");
    assert_eq!(flatted_text[index + 5], "failed");
}

#[test]
fn parallel_text_non_sequintial_tokenization() {
    let texts = ["jump to content failed failed failed", "contact us"];
    let flatted_text: Vec<String> = texts
        .into_par_iter()
        .flat_map(clean_and_tokenize2)
        .collect();
    let (index, value) = flatted_text
        .iter()
        .enumerate()
        .find(|(x, e)| **e == "jump")
        .unwrap();
    println!("{:?}", flatted_text);
    assert_eq!(flatted_text[index + 1], "to");
    assert_eq!(flatted_text[index + 2], "content");
    assert_eq!(flatted_text[index + 3], "failed");
    assert_eq!(flatted_text[index + 4], "failed");
    assert_eq!(flatted_text[index + 5], "failed");
}
//-------------------------test mutatuing an index from another thread;
#[tokio::test]
async fn test_variable_mutation() {
    let index = Arc::new(Mutex::new(0));
    let copied = index.clone();
    tokio::spawn(async move {
        for i in 0..5 {
            *copied.clone().lock().unwrap() += 1;
        }
    })
    .await
    .expect("failed to calculate");
    assert_eq!(*index.lock().unwrap(), 5);
}
#[tokio::test]
async fn test_variable_mutation2() {
    let index = Arc::new(AtomicUsize::new(0));
    let copied = index.clone();
    tokio::spawn(async move {
        for i in 0..5 {
            copied.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    })
    .await
    .expect("failed to count");
    assert_eq!(index.load(std::sync::atomic::Ordering::SeqCst), 5);
}

//-------------------------------------dashmap testing------------------

#[tokio::test]
async fn dash_map() {
    let mut dashy = DashMap::new();
    dashy.insert(&5, vec![5]);
    dashy.entry(&5).or_insert_with(Vec::new).push(7);
    dbg!(&dashy);
}
#[tokio::test]
async fn inverted_test() {
    let dashy = DashMap::new();
    let key = Arc::new("shawarma".to_owned());
    let mut index = Invindex::new(InvindexId {
        term: key.clone(),
        doc_url: "http://potato.net".to_owned(),
    });
    index.add_location(77); //insetes basic values
    dashy.insert(index.get_term(), index.clone()); //inserts key:value
    dashy
        .entry(index.get_term()) //checks whether a value exists and return a mutable refernce for it
        .or_insert(index)
        .add_location(5);
    let indox = dashy.get(&key);
    dbg!(&dashy);
    assert_eq!(indox.unwrap().get_locations(), &[77, 5]);
}
//-------------------------------------database tests------------------

#[tokio::test]
async fn connect_base() {
    let db = get_db_connection().await;

    dbg!(&db);
    db.query("CREATE inv_index:{term:'shlawii',doc_url:'http://flafel.net'} SET location=[1,2,3],doc_length=5,tf=0.99;").await;
}

#[tokio::test]
async fn insert_base() -> CResult<()> {
    let id = InvindexId {
        term: Arc::new("term1".into()),
        doc_url: "https://flafe.net".into(),
    };
    let mut entry = Invindex::new(id).location(vec![1, 2, 3]);
    entry.doc_length(30);

    let value: surrealdb::Result<Option<ItemId>> = get_db_connection()
        .await
        .create("inv_index")
        .content(entry)
        .await;
    assert!(!value.is_err());
    Ok(())
}
//this for inserting records
async fn upsert_base() -> CResult<()> {
    let id = InvindexId {
        term: Arc::new("term1".into()),
        doc_url: "https://flafel.net".into(),
    };
    let mut entry = Invindex::new(id).location(vec![1, 2, 3, 4, 5, 7]);
    entry.doc_length(35);

    let value: surrealdb::Result<Vec<ItemId>> = get_db_connection()
        .await
        .upsert("inv_index")
        .content(entry)
        .await;
    //assert!(value.is_ok());
    Ok(())
}

async fn insert_url_doc(url: &str) {
    let db = get_db_connection().await;
    let value: surrealdb::Result<Vec<ItemId>> = db
        .upsert("document")
        .content(Document {
            id: DocumentId {
                url: url.to_string(),
            },
        })
        .await;
    assert!(value.is_ok());
}

async fn create_docs() {
    let mut url = "https://potato.net".to_string();
    for i in 0..5 {
        url.push_str(&format!("{}{}", url, i));
        insert_url_doc(&url).await;
    }
}
//--------------------------testing the deserilaization----------------
#[tokio::test]
async fn retrive_data() {
    let _ = upsert_base().await;
    let expected = TermDocRecord {
        id: TermDocRecordId {
            term: Rc::new("term1".into()),
            doc_url: Rc::new("https://flafe.net".into()),
        },
        frequency: 6,
        doc_length: 35,
        tf_idf: 0.0,
        tf: 0.17142857142857143,
    };
    let db = get_db_connection().await;
    let values: Vec<TermDocRecord> = db
        .query("SELECT id.term,id.doc_url,frequency,doc_length,tf FROM inv_index WHERE id.term='term1' ;")
        .await
        .unwrap()
        .take(0)
        .unwrap();

    assert_eq!(values[0], expected);
}

#[tokio::test]
async fn retrive_not_exist() {
    let db = get_db_connection().await;
    let values: Vec<TermDocRecord> = db
        .query("SELECT id.term,id.doc_url,doc_length,tf FROM inv_index WHERE id.term='balah' ;")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    assert!(values.is_empty());
}

async fn total_num(query: &str) -> surrealdb::Result<Vec<usize>> {
    get_db_connection()
        .await
        .query(query)
        .await
        .unwrap()
        .take(0)
}

#[tokio::test]
async fn total_num_dcoument() {
    let query = "SELECT VALUE count() FROM document;";
    let total_doc = total_num(query).await;

    assert!(total_doc.is_ok());
    assert_eq!(total_doc.unwrap().len(), 0);
}

#[tokio::test]
async fn total_num_doc() {
    create_docs().await;
    let query = "RETURN( select Value count() FROM document GROUP ALL )[0].count OR 0;";
    let total_doc = total_num(query).await;
    dbg!(&total_doc);
    assert!(total_doc.is_ok());
    assert_eq!(total_doc.unwrap()[0], 5);
}
#[tokio::test]
async fn total_num_doc_zero() {
    //create_docs().await;
    let query = "RETURN( select Value count() FROM document GROUP ALL )[0].count OR 0;";
    let total_doc = total_num(query).await;
    dbg!(&total_doc);
    assert!(total_doc.is_ok());
    assert_eq!(total_doc.unwrap()[0], 0);
}

#[tokio::test]
async fn total_num_inv_index() {
    let query = "SELECT VALUE count() FROM inv_index;";
    let total_doc = total_num(query).await;

    assert!(total_doc.is_ok());
    assert!(total_doc.unwrap()[0] > 0);
}
fn make_doc(freq: usize, tf_idf: f64) -> TermDocRecord {
    TermDocRecord {
        id: TermDocRecordId {
            term: Rc::new("hello".to_string()),
            doc_url: Rc::new("flafel".to_string()),
        },
        doc_length: 5,
        frequency: freq,
        tf_idf,
        tf: 5.4,
    }
}
#[tokio::test]
async fn compare_docs() {
    let doc1 = make_doc(5, 0.0);
    let doc2 = make_doc(8, 0.0);
    assert_eq!(doc1, doc2);
}

#[tokio::test]
async fn number_docs_hashed() {
    let mut docs = HashSet::new();
    for x in 0..10 {
        docs.insert(make_doc(x, 0.0));
    }
    assert_eq!(docs.len(), 1);
}

#[tokio::test]
async fn pop_from_hash() {
    let doc = make_doc(8, 0.0);
    let mut table = HashSet::new();
    table.insert(doc);
}

#[tokio::test]
async fn check_add() {
    let doc1 = make_doc(9, 1.0);
    let doc2 = make_doc(5, 2.0);
    let summed = doc1 + doc2;
    assert_eq!(summed.tf_idf, 3.0);
    assert_eq!(summed.frequency, 9);
}
//-------------------------------testing unify--------------------
fn create_table(docs: Vec<TermDocRecord>) -> HashMap<Rc<String>, TermDocRecord> {
    let mut table = HashMap::new();
    for item in docs.into_iter() {
        table.insert(item.get_url(), item);
    }
    table
}

#[test]
fn test_unify_doc() {
    let expected = make_doc(55, 6.0);
    let doc1 = make_doc(55, 1.0);
    let doc2 = make_doc(54, 2.0);
    let doc3 = make_doc(89, 3.0);
    let table = create_table(vec![doc1]);
    let table2 = create_table(vec![doc2]);
    let table3 = create_table(vec![doc3]);

    let unified_docs = unify_docs(vec![table, table2, table3]);
    assert_eq!(expected, *unified_docs.get(&expected.get_url()).unwrap());
    assert_eq!(
        expected.tf_idf,
        unified_docs.get(&expected.get_url()).unwrap().tf_idf
    );
    assert_eq!(unified_docs.len(), 1);
    assert_eq!(unified_docs.get(&expected.get_url()).unwrap().tf_idf, 6.0);
}
//------------------------testing ordering a vec of TermDocRecord---------------

#[test]
fn sorting_documents_tf_idf() {
    let doc1 = make_doc(5, 1.0);
    let doc2 = make_doc(5, 2.0);
    let doc3 = make_doc(5, 3.0);
    let doc4 = make_doc(5, 4.0);
    let mut table = [&doc4, &doc3, &doc2, &doc1];
    table.sort();
    assert_eq!(table, [&doc1, &doc2, &doc3, &doc4]);
    assert_eq!(table[0].tf_idf, 1.0);
}
//-------------------testing file logs creation--------------

fn create_log_file(kind_file: &str) -> BufWriter<File> {
    let exe_path = env::current_exe().unwrap();
    let log_dir = exe_path.parent().unwrap().join("logs");
    if !log_dir.exists() {
        // creates the logs directory
        create_dir_all(&log_dir).unwrap();
    }
    let log = log_dir.join(kind_file);
    let file = File::create(log).unwrap();
    BufWriter::new(file)
}

#[test]
fn create_log_success() {
    let mut buf = create_log_file("log_suc.txt");

    assert!(write!(&mut buf, "hello").is_ok());
}
