use super::TermDocRecord;
use crate::{get_db_connection, unify_docs};
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

/// gets all records that is related to our query ;
pub async fn qurey_database(query: &str) -> Option<Vec<TermDocRecord>> {
    let query_tokenized = tokenize_query(query);
    let number_documents = get_number_documents_stored().await as f64;
    let words_docs = get_words_docs(query_tokenized, number_documents).await;
    if words_docs.is_empty() {
        return None;
    }
    let doc_terms = unify_docs(words_docs);
    let mut list_docs: Vec<TermDocRecord> = doc_terms.into_values().collect();
    list_docs.sort();
    Some(list_docs)
}

/// returns a list of tables that consist of doc_url : TermDoc container
///
/// consumes the query tokens (words) !!
async fn get_words_docs(
    query_tokenized: HashSet<String>,
    total_docs: f64,
) -> Vec<HashMap<Rc<String>, TermDocRecord>> {
    let db = get_db_connection().await;
    let mut term_doc = vec![];
    for word in query_tokenized {
        let mut response = db
            .query(QUERY)
            .bind(("word", word.clone()))
            .await
            .expect("failed to execute query");
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

/// cleans and tokinizes the query text
///
pub fn tokenize_query(query: &str) -> HashSet<String> {
    query
        .split(|c: char| !c.is_alphabetic())
        .filter(|token| token.len() > 1)
        .map(move |token| token.to_lowercase())
        .collect()
}

/// returns the number of doc_url stored in the whole database!!
async fn get_number_documents_stored() -> usize {
    let tota_num_documents =
        "RETURN( SELECT VALUE count() FROM document GROUP ALL )[0].count OR 0;";
    let total_num: Vec<usize> = get_db_connection()
        .await
        .query(tota_num_documents)
        .await
        .unwrap()
        .take(0)
        .unwrap_or(vec![0]);
    *total_num.first().unwrap()
}
