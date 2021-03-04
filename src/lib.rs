pub mod graph;
pub mod grid;
pub mod logging;
pub mod generator;

pub fn sanitise_string(string: &str, allowed_chars: &str) -> String {
    let sanitised = string.replace(|c: char| allowed_chars.find(c).is_none(), "");
    sanitised
}
