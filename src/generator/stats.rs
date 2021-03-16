use std::collections::HashMap;
use std::iter::Sum;
use std::hash::Hash;

fn mean<T, I: Iterator<Item=T>>(iter_values: I) -> Option<f64>
where
    T: Into<f64> + Sum<T>,
{
    let mut len = 0;
    let sum: T = iter_values.map(|t| {
        len += 1;
        t
    }).sum::<T>();
    match len {
        0 => None,
        _ => Some(sum.into() / len as f64)
    }
}

use super::CrosswordGridScore;

pub fn mean_of_hashmaps<T, U>(hashmaps: Vec<HashMap<U, T>>) -> HashMap<U, f64>
where
    T: Into<f64> + Sum<T> + Copy,
    U: Eq + Hash + Copy,
{
    let mut means: HashMap<U, f64> = HashMap::new();
    for key in hashmaps[0].keys() {
        let all_values: Vec<T> = hashmaps.iter().map(|h| *h.get(key).unwrap()).collect();
        means.insert(*key, mean(all_values.into_iter()).unwrap());
    }
    means
}

impl CrosswordGridScore {
    pub fn average_scores(scores: Vec<Self>) -> Self {
        let hashmaps: Vec<HashMap<String, f64>> = scores.iter().map(CrosswordGridScore::to_hashmap).collect();
        let mut means: HashMap<String, f64> = HashMap::new();

        for key in hashmaps[0].keys() {
            let all_values: Vec<f64> = hashmaps.iter().map(|h| *h.get(key).unwrap()).collect();
            means.insert(key.to_string(), mean(all_values.into_iter()).unwrap());
        }
        let v = serde_json::to_value(means).unwrap();
        serde_json::from_value(v).unwrap()
    }

    fn to_hashmap(&self) -> HashMap<String, f64> {
        let v = serde_json::to_value(self).unwrap();
        serde_json::from_value(v).unwrap()
    }
}
