extern crate radix_trie;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::iter::Sum;
use std::iter::{Chain, Enumerate};
use std::ops::{Deref, Index};
use std::slice::Iter;

// TODO: make this not allocate
#[derive(Deserialize)]
pub struct Dictionary(Vec<String>);

impl Deref for Dictionary {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct HashedDictionary<'a>(HashSet<&'a [u8]>);

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

pub struct Grid {
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
    pub fn is_valid(&self, dict: &HashedDictionary) -> bool {
        dict.contains(&self.s4 as &[u8]) && dict.contains(&self.s5 as &[u8])
            && dict.contains(&self.s6 as &[u8])
    }

    fn slot_iter(&self) -> SlotIter {
        SlotIter(
            self.s1
                .iter()
                .chain(self.s2.iter())
                .chain(self.s3.iter())
                .enumerate(),
        )
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

// returns the slot, index 0 being the top left, and 8 being the bottom right
impl Index<usize> for Grid {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < 9);

        match index {
            0 | 1 | 2 => &self.s1[index],
            3 | 4 | 5 => &self.s2[index - 3],
            6 | 7 | 8 => &self.s3[index - 6],
            _ => panic!("bad value"),
        }
    }
}

struct SlotIter<'a>(Enumerate<Chain<Chain<Iter<'a, u8>, Iter<'a, u8>>, Iter<'a, u8>>>);

impl<'a> Iterator for SlotIter<'a> {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        // dereference the &u8 char to an u8
        self.0.next().map(|(i, c)| (i, *c))
    }
}

pub struct GridLetterCount([[usize; 26]; 9]);

impl GridLetterCount {
    pub fn new() -> Self {
        GridLetterCount([[0; 26]; 9])
    }

    pub fn increment(&mut self, grid: &Grid) {
        // for all the characters and they're indexes, increment the associated count
        for (i, c) in grid.slot_iter() {
            self.0[i][(c - b'a') as usize] += 1;
        }
    }

    fn merge(&mut self, grid_count: GridLetterCount) {
        for (mut this_slot, that_slot) in self.0.iter_mut().zip(grid_count.0.iter()) {
            for (this_count, that_count) in this_slot.iter_mut().zip(that_slot.iter()) {
                *this_count += that_count;
            }
        }
    }
}

impl Display for GridLetterCount {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        for (si, slot) in self.0.iter().enumerate() {
            write!(f, "slot{}: ", si)?;
            for (c, count) in slot.iter().enumerate() {
                write!(f, "{}({:7}),", char::from(c as u8 + b'a'), count)?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

impl Sum for GridLetterCount {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = GridLetterCount>,
    {
        let mut grid_count = GridLetterCount::new();
        for that_count in iter {
            grid_count.merge(that_count);
        }

        grid_count
    }
}
