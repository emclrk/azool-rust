use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Clone, Copy, Debug, Display, EnumIter, Eq, Hash, PartialEq)]
enum TileColor {
    RED,
    BLUE,
    GREEN,
    YELLOW,
    WHITE,
}

const NUM_COLORS: i32 = 5;
const NUM_TILES_PER_COLOR: i32 = 20;
fn reset_board(tile_bag: &mut Vec<TileColor>) -> &mut Vec<TileColor> {
    tile_bag.clear();
    tile_bag.reserve((NUM_COLORS * NUM_TILES_PER_COLOR).try_into().unwrap());
    for color in TileColor::iter() {
        for _ii in 0..NUM_TILES_PER_COLOR {
            tile_bag.insert(0, color);
        }
    }
    tile_bag
}  // reset_board

struct Player {
    my_score : i32,
    my_num_penalties_for_round : i32,
    my_took_pool_penalty_this_round : bool,
    my_name : String,
}

impl Player {
    fn check_valid_move(&self, color : &TileColor, row_idx : &i32) -> bool {
        true
    }
    fn score_tile(&self, row_idx : &i32, col_idx :& i32) -> i32 {
        0
    }
}

fn main() {
    // let num_factories = 4;
    // let mut factory_vec = Vec::new();
    let mut pool = HashMap::new();
    let mut tile_bag = Vec::new();
    reset_board(&mut tile_bag);
    for color in TileColor::iter() {
        pool.insert(color, 0);
    }
}
