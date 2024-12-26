use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::{Add, AddAssign};
use std::rc::Rc;
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
pub struct TermDocRecordId {
    pub term: Rc<String>,
    pub doc_url: Rc<String>,
}

fn default_zero() -> f64 {
    0.0
}

#[derive(Deserialize, Debug)]
pub struct TermDocRecord {
    pub id: TermDocRecordId,
    pub doc_length: usize,
    pub frequency: usize,
    #[serde(default = "default_zero")]
    pub tf_idf: f64,
    pub tf: f64,
}
impl AddAssign for TermDocRecord {
    fn add_assign(&mut self, rhs: Self) {
        self.tf_idf += rhs.tf_idf;
    }
}
impl Add for TermDocRecord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        TermDocRecord {
            tf_idf: self.tf_idf + rhs.tf_idf,
            ..self
        }
    }
}

impl Ord for TermDocRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.tf_idf
            .partial_cmp(&other.tf_idf)
            .unwrap_or(Ordering::Equal)
    }
}
impl PartialOrd for TermDocRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl TermDocRecord {
    pub fn set_tf_idf(&mut self, idf: f64) {
        self.tf_idf = idf;
    }
    pub fn get_url(&self) -> Rc<String> {
        self.id.doc_url.clone()
    }
    pub fn get_term(&self) -> Rc<String> {
        self.id.term.clone()
    }
}
impl Eq for TermDocRecord {}
impl Hash for TermDocRecord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Hash for TermDocRecordId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.doc_url.hash(state);
    }
}
impl PartialEq for TermDocRecord {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialEq for TermDocRecordId {
    fn eq(&self, other: &Self) -> bool {
        self.doc_url == other.doc_url
    }
}
