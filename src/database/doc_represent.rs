use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::Arc;
use surrealdb::RecordId;

#[derive(Serialize, Debug, Eq, Clone)]
pub struct Invindex {
    id: InvindexId,
    doc_length: usize,
    location: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone)]
pub struct InvindexId {
    pub term: Arc<String>,
    pub doc_url: String,
}
impl InvindexId {
    fn get_term(&self) -> Arc<String> {
        self.term.clone()
    }
}
impl Invindex {
    pub fn new(id: InvindexId) -> Self {
        Self {
            id,
            doc_length: 0,

            location: vec![],
        }
    }
    pub fn get_term(&self) -> Arc<String> {
        self.id.get_term()
    }
    pub fn location(mut self, locs: Vec<usize>) -> Self {
        self.location = locs;
        self
    }
    pub fn doc_length(&mut self, leng: usize) {
        self.doc_length = leng;
    }
    pub fn add_location(&mut self, loc: usize) {
        self.location.push(loc);
    }
    pub fn get_locations(&self) -> &Vec<usize> {
        &self.location
    }
}

impl Hash for InvindexId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.term.hash(state);
    }
}
impl PartialEq for InvindexId {
    fn eq(&self, other: &Self) -> bool {
        self.term == other.term
    }
}

impl Hash for Invindex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for Invindex {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentId {
    pub url: String,
}

impl DocumentId {
    pub fn get_url(&self) -> String {
        self.url.clone()
    }
}
#[derive(Deserialize, Debug)]
pub struct ItemId {
    id: RecordId,
}

#[derive(Deserialize, Debug)]
pub struct TermDocRecord {
    pub id: InvindexId,
    pub doc_length: usize,
    pub tf: f64,
}
