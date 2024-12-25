use super::TermDocRecord;
use crate::get_db_connection;
use libm::log10;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

static QUERY: &str =
    "SELECT id.term,id.doc_url,frequency,doc_length,tf FROM inv_index WHERE id.term=$word";

fn calculate_idf(total_docs: f64, df: f64) -> f64 {
    log10(total_docs / (1.0 + df))
}

pub async fn qurey_database(query: &str) {
    let query_tokenized = tokenize_query(query);
    let number_documents = get_number_documents_stored().await as f64;
    let words_docs = get_words_docs(query_tokenized, number_documents).await;
    todo!("calling unify_docs and ranking the results , writing ordering for TermDocId")
}

/// returns a list of tables that consist of doc_url : TermDoc container
async fn get_words_docs(
    query_tokenized: HashSet<String>,
    total_docs: f64,
) -> Vec<HashMap<Rc<String>, TermDocRecord>> {
    let db = get_db_connection().await;
    let mut term_doc = vec![];
    for word in query_tokenized {
        let mut response = db.query(QUERY).bind(("word", word.clone())).await.unwrap();
        let records: Vec<TermDocRecord> = response.take(0).unwrap();
        if !records.is_empty() {
            let df = records.len() as f64;
            // if a query term is not in the database , its rarely happens
            let mut table = HashMap::new();
            records.into_iter().for_each(|mut term| {
                term.set_tf_idf(calculate_idf(total_docs, df) * term.tf);
                table.insert(term.get_url(), term);
            });
            term_doc.push(table);
        }
    }
    term_doc
}

fn tokenize_query(query: &str) -> HashSet<String> {
    query
        .split(|c: char| !c.is_alphabetic())
        .filter(|token| token.len() > 1)
        .map(move |token| token.to_owned())
        .collect()
}
async fn get_number_documents_stored() -> usize {
    let tota_num_documents = "SELECT count() FROM document;";
    let total_num: surrealdb::Result<Vec<usize>> = get_db_connection()
        .await
        .query(tota_num_documents)
        .await
        .unwrap()
        .take(0);
    match total_num {
        Ok(v) => {
            if !v.is_empty() {
                v[0]
            } else {
                0
            }
        }
        _ => 0,
    }
}
