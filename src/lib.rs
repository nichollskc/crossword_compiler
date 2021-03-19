use std::hash::Hash;
use std::collections::HashMap;

#[macro_use]
extern crate ndarray;

#[macro_use]
extern crate matches;

#[macro_use] extern crate lazy_static;
extern crate regex;

pub mod graph;
pub mod grid;
pub mod logging;
pub mod generator;
pub mod utils;

pub fn sanitise_string(string: &str, allowed_chars: &str) -> String {
    let sanitised = string.replace(|c: char| allowed_chars.find(c).is_none(), "");
    sanitised
}

pub fn custom_hashmap_format<U, T>(hashmap: &HashMap<U, T>,
                                   key_prefix: &str,
                                   delimiter: &str) -> String
where
    U: std::fmt::Debug + Eq + Hash,
    T: std::fmt::Debug,
{
    let mut result = String::new();
    result.push_str("(( ");
    for (key, value) in hashmap.iter() {
        result.push_str(&format!("{}{:#?}{}{:#?}, ",
                                key_prefix,
                                key,
                                delimiter,
                                value));
    }
    result.push_str("))");
    result
}
