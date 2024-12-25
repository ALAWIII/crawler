use crate::TermDocRecord;
use std::collections::HashMap;
use std::rc::Rc;

type T = HashMap<Rc<String>, TermDocRecord>;

pub fn unify_docs(mut words_docs: Vec<T>) -> T {
    let mut word = words_docs.pop().unwrap();

    while let Some(w) = words_docs.pop() {
        combine_docs(&mut word, w);
    }
    word
}

fn combine_docs(w1: &mut T, w2: T) {
    for (k, v) in w2 {
        if let Some(term_doc) = w1.get_mut(&k) {
            *term_doc += v; // Use Add implementation
        } else {
            w1.insert(k, v); // Insert new key-value pair
        }
    }
}
