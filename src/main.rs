extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

use rayon::prelude::*;

const DICTIONARY: &str = include_str!("../resources/dictionary.json");
//const DICTIONARY: &str = "[\"aaa\",\"aaa\",\"aaa\"]";

// TODO: make this not allocate
#[derive(Deserialize)]
struct Dictionary(Vec<String>);

impl Deref for Dictionary {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

struct HashedDictionary<'a>(HashSet<&'a [u8]>);

impl<'a> HashedDictionary<'a> {
    pub fn from_set(set: HashSet<&'a [u8]>) -> Self {
        HashedDictionary(set)
    }
}

impl<'a> Deref for HashedDictionary<'a> {
    type Target = HashSet<&'a [u8]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
struct Grid {
    s1: [u8; 3],
    s2: [u8; 3],
    s3: [u8; 3],
    s4: [u8; 3],
    s5: [u8; 3],
    s6: [u8; 3],
}

impl Grid {
    /// The strs must be valid strings
    pub fn from_strs(s1: &[u8], s2: &[u8], s3: &[u8]) -> Self {
        let mut grid = Grid {
            s1: [0u8; 3],
            s2: [0u8; 3],
            s3: [0u8; 3],
            s4: [0u8; 3],
            s5: [0u8; 3],
            s6: [0u8; 3],
        };

        grid.s1.copy_from_slice(s1);
        grid.s2.copy_from_slice(s2);
        grid.s3.copy_from_slice(s3);
        grid.s4.copy_from_slice(&Self::to_array(s1, s2, s3, 0));
        grid.s5.copy_from_slice(&Self::to_array(s1, s2, s3, 1));
        grid.s6.copy_from_slice(&Self::to_array(s1, s2, s3, 2));

        grid
    }

    #[inline(always)]
    fn to_array(s1: &[u8], s2: &[u8], s3: &[u8], idx: usize) -> [u8; 3] {
        [s1[idx], s2[idx], s3[idx]]
    }

    /// checks that only the last three constructed strings are valid, the initial strings passed in are assumed to be valid
    fn is_valid(&self, dict: &HashedDictionary) -> bool {
        dict.contains(&self.s4 as &[u8]) && dict.contains(&self.s5 as &[u8])
            && dict.contains(&self.s6 as &[u8])
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "-----")?;
        writeln!(f, "|{}|", String::from_utf8_lossy(&self.s1),)?;
        writeln!(f, "|{}|", String::from_utf8_lossy(&self.s2),)?;
        writeln!(f, "|{}|", String::from_utf8_lossy(&self.s3),)?;
        writeln!(f, "-----")?;

        Ok(())
    }
}

fn main() {
    // TODO: with a custom parser, we could have refs to &'static str
    let dict: Dictionary = serde_json::from_str(DICTIONARY).expect("failed to parse dictionary");

    println!("read this many words: {}", dict.len());

    // collect the dictionary
    let three_letter_dict: HashedDictionary = HashedDictionary::from_set(
        dict.iter()
            .filter(|s| s.len() == 3)
            .map(|s| s.as_bytes())
            .collect(),
    );

    println!(
        "read this many three letter words: {}",
        three_letter_dict.len()
    );

    // collect the grids
    let grids: Vec<Grid> = three_letter_dict
        .par_iter()
        .map(|s1| {
            let mut grids = Vec::<Grid>::with_capacity(10_000);
            for s2 in three_letter_dict.iter() {
                for s3 in three_letter_dict.iter() {
                    let grid = Grid::from_strs(s1, s2, s3);

                    if grid.is_valid(&three_letter_dict) {
                        grids.push(grid)
                    }
                }
            }

            grids
        })
        .flatten()
        .collect();

    println!("found this many valid grids: {}", grids.len());

    // for i in 0..50 {
    //     println!("{}", grids[i]);
    // }
}
