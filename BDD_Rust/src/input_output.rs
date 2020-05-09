use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

const FILE_PATH: &'static str = "input_data/input2.json";
// path for JSON file

#[derive(Clone)]
pub struct RootList {
  // struct so save Roots
  pub roots: Vec<String>,
}


#[derive(Serialize, Deserialize)]
struct Input {
  paths: Vec<PathInformation>,
}

#[derive(Serialize, Deserialize, Clone)]
struct PathInformation {
  // struct so save relevant path information
  from: String,
  to: String,
  costs: u16,
}
pub fn input_parse(rootlist: &mut RootList, data: String) -> Result<()> {
  // Parse the string of data into a Input object
  let input: Input = serde_json::from_str(&data)?;
  let mut root = HashSet::new();
  for roots in &input.paths {
    // using Hashset for extracting start nodes
    root.insert(roots.from.clone());
    root.insert(roots.to.clone());
  }

  for roots in &root {
    rootlist.roots.push(roots.to_string());   // give the startnode list back
  }

  Ok(()) 
}

pub fn read_input() -> String {
  let mut file = File::open(FILE_PATH).expect("could not open file");
  let mut file_input = String::new();
  file
    .read_to_string(&mut file_input)
    .expect("could not read file");
  return file_input;
}
