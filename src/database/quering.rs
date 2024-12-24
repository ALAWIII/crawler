use crate::get_db_connection;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::TermDocRecord;
static QUERY: &str = "SELECT id,doc_length,tf FROM inv_index WHERE id.term=$word";

pub async fn qurey_database(query: &str) {
    let db = get_db_connection().await;
    let segmented = tokenize_query(query);
    let mut term_doc = HashMap::new();
    for word in segmented {
        let response = db.query(QUERY).bind(("word", word.clone())).await.unwrap();

        term_doc.insert(word, response);
    }
}

fn tokenize_query(query: &str) -> HashSet<Arc<String>> {
    query
        .split(|c: char| !c.is_alphabetic())
        .filter(|token| token.len() > 1)
        .map(move |token| Arc::new(token.to_owned()))
        .collect()
}
