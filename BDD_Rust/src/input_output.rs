use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
// Type inference lets us omit an explicit type signature (which
// would be `HashSet<String>` in this example).

const FILE_PATH: &'static str = "input_data/input2.json";

#[derive(Clone)]
pub struct RootList {
  pub roots: Vec<String>,
}

#[derive(Clone)]
pub struct Rootstring {
  // struct so save relevant path information
  string: String,
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
pub fn input_parse(Roots: &mut RootList, data: String) -> Result<()> {
  // Parse the string of data into a Input object
  let input: Input = serde_json::from_str(&data)?;
  let mut Root = HashSet::new();
  for roots in &input.paths {
    Root.insert(roots.from.clone());
    Root.insert(roots.to.clone());
  }

  for roots in &Root {
    Roots.roots.push(roots.to_string());
  }

  // Do things just like with any other Rust data structure.
  //println!("Distance from {} to {} is {}", input.paths[0].from, input.paths[0].to, input.paths[0].costs);
  //println!("Distance from {} to {} is {}", input.paths[1].from, input.paths[1].to, input.paths[1].costs);
  // Access parts of the data by indexing with square brackets.

  Ok(()) // returning root list
}

pub fn read_input() -> String {
  let mut file = File::open(FILE_PATH).expect("could not open file");
  let mut file_input = String::new();
  file
    .read_to_string(&mut file_input)
    .expect("could not read file");
  return file_input;
}

pub fn show_input(input: &String) {
  println!(" Json Input: \n\n{}", input);
}
