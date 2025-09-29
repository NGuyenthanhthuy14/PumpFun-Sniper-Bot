use itertools::Itertools;
use std::io::BufRead;
use std::{fs, io};

#[derive(Clone, Debug)]
pub struct BlackList {
    addresses: Vec<String>,
}

impl BlackList {
    pub fn get_blacklist(file_path: &str) -> Vec<String> {
        let mut blacklist: Vec<String> = Vec::new();

        let file = fs::File::open(file_path).unwrap();
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line_string = line.unwrap();
            blacklist.push(line_string);
        }

        blacklist.into_iter().unique().collect()
    }

    pub fn get_length(&mut self) -> usize {
        self.addresses.iter().len()
    }

    pub fn is_blacklisted(&self, address: &str) -> bool {
        self.addresses.contains(&address.to_string())
    }
}
