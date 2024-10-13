use std::collections::{HashMap, VecDeque};
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use regex::Regex;

fn inv_dict_list(dictionnary: &HashMap<String, Vec<String>>) -> HashMap<String, String> {
    let mut inv_d = HashMap::new();
    for (key1, value_list) in dictionnary {
        for key2 in value_list {
            inv_d.insert(key2.clone(), key1.clone());
        }
    }
    inv_d
}

pub struct Queue<T> {
    inner: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            inner: VecDeque::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        self.inner.push_front(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_back()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

pub struct Stack<T> {
    inner: VecDeque<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack {
            inner: VecDeque::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        self.inner.push_back(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_back()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

pub fn sub(chaine: &str, dico: &HashMap<String, String>) -> String {
    let pattern = Regex::new(&format!("({})", dico.keys().cloned().collect::<Vec<_>>().join("|"))).unwrap();
    pattern.replace_all(chaine, |caps: &regex::Captures| {
        dico.get(&caps[0]).unwrap_or(&caps[0]).to_string()
    }).to_string()
}

// Huffman tree generation
pub fn huffman(ctr: &HashMap<String, usize>) -> Vec<(String, String)> {
    let mut forest: BinaryHeap<Reverse<(usize, Vec<(String, String)>)>> = BinaryHeap::new();

    for (key, &freq) in ctr {
        forest.push(Reverse((freq, vec![("".to_string(), key.clone())])));
    }

    if forest.is_empty() {
        return vec![];
    }

    if forest.len() == 1 {
        let Reverse((_, mut single_tree)) = forest.pop().unwrap();
        single_tree[0].0 = "0".to_string();
        return single_tree;
    }

    while forest.len() > 1 {
        let Reverse((freq_x, left_tree)) = forest.pop().unwrap();
        let Reverse((freq_y, right_tree)) = forest.pop().unwrap();

        let new_freq = freq_x + freq_y;
        let new_tree: Vec<_> = left_tree.into_iter().map(|(pos, key)| ("0".to_string() + &pos, key))
            .chain(right_tree.into_iter().map(|(pos, key)| ("1".to_string() + &pos, key)))
            .collect();

        forest.push(Reverse((new_freq, new_tree)));
    }

    let Reverse((_, tree)) = forest.pop().unwrap();
    tree = tree.into_iter().sorted_by_key(|(pos, _)| pos.len()).collect();
    tree
}