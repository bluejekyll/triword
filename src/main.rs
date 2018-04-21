extern crate radix_trie;
extern crate rayon;
extern crate serde;
extern crate serde_json;
extern crate triword;

use radix_trie::Trie;
use rayon::prelude::*;
use triword::*;

//const DICTIONARY: &str = include_str!("../resources/dictionary.json");
const DICTIONARY: &str = include_str!("../resources/twl3.txt");
const ALL_THREES: &str = include_str!("../resources/all-threes.txt");
//const DICTIONARY: &str = "[\"aaa\",\"aaa\",\"aaa\"]";

fn parse_txt_dict() -> Vec<&'static str> {
    DICTIONARY
        .split("\n")
        .chain(ALL_THREES.split("\n"))
        .collect()
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .stack_size(8 * 1024 * 1024)
        .build_global()
        .unwrap();

    // TODO: with a custom parser, we could have refs to &'static str
    //let dict: Dictionary = serde_json::from_str(DICTIONARY).expect("failed to parse dictionary");
    let dict = parse_txt_dict();

    println!("read this many words: {}", dict.len());

    // collect the dictionary
    let unique_dict: HashedDictionary = HashedDictionary::from_set(
        dict.iter()
            .filter(|s| s.len() == 3)
            .map(|s| s.as_bytes())
            .collect(),
    );
    let trie_dict: Trie<&[u8], &[u8]> = unique_dict.iter()
        // mapping to a tuple of bytes to bytes
        .map(|s| (*s, *s))
        .collect();

    println!("read this many three letter words: {}", unique_dict.len(),);

    // collect the grids
    let grids: Vec<Grid> = unique_dict
        .par_iter()
        .map(|s1| {
            let mut grids = Vec::<Grid>::with_capacity(10_000);
            for s2 in unique_dict.iter() {
                let ps1 = [s1[0], s2[0]];
                let ps2 = [s1[1], s2[1]];
                let ps3 = [s1[2], s2[2]];

                if trie_dict.subtrie(&ps1 as &[u8]).is_none() {
                    continue;
                }
                if trie_dict.subtrie(&ps2 as &[u8]).is_none() {
                    continue;
                }
                if trie_dict.subtrie(&ps3 as &[u8]).is_none() {
                    continue;
                }

                for s3 in unique_dict.iter() {
                    let grid = Grid::from_strs(s1, s2, s3);

                    if grid.is_valid(&unique_dict) {
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

    let grid_count: GridLetterCount = grids
        .par_iter()
        .fold(GridLetterCount::new, |mut grid_count, grid| {
            grid_count.increment(grid);
            grid_count
        })
        .sum();

    println!("slot counts:\n{}", grid_count);

    let try_slots = [(5, b'j'), (1, b'x'), (6, b't')];

    let mut grid_count = GridLetterCount::new();
    for grid in grids
        .iter()
        .filter(|grid| try_slots.iter().all(|(slot, ch)| grid[*slot] == *ch))
    {
        grid_count.increment(grid);
    }

    println!("{:?} counts:\n{}", try_slots, grid_count);


}
