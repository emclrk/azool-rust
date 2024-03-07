#![allow(dead_code)]
#![allow(unused_mut)] // TODO: remove this when done
use std::collections::HashMap;
// use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Clone, Copy, Debug, Display, EnumIter, Eq, Hash, PartialEq)]
enum TileColor {
    NOCOLOR = -1,
    RED = 0,
    BLUE = 1,
    GREEN = 2,
    YELLOW = 3,
    WHITE = 4,
    NUMCOLORS = 5,
}

impl TileColor {
    pub fn into(&self) -> i32 {
        match self {
            TileColor::NOCOLOR => -1,
            TileColor::RED => 0,
            TileColor::BLUE => 1,
            TileColor::GREEN => 2,
            TileColor::YELLOW => 3,
            TileColor::WHITE => 4,
            TileColor::NUMCOLORS => 5,
        }
    }
    pub fn from(input: usize) -> Self {
        match input {
            0 => TileColor::RED,
            1 => TileColor::BLUE,
            2 => TileColor::GREEN,
            3 => TileColor::YELLOW,
            4 => TileColor::WHITE,
            5 => TileColor::NUMCOLORS,
            _ => TileColor::NOCOLOR,
        }
    }
}
enum InvalidMoveError {
    BADCOLOR,
}

const NUM_TILES_PER_COLOR: i32 = 20;
const PENALTY_POINTS: [i32; 9] = [0, 1, 2, 3, 5, 7, 10, 13, 15];
#[derive(Debug)]
struct Factory {
    // instead maybe use hashmap color->count
    num_red: i32,
    num_green: i32,
    num_blue: i32,
    num_yellow: i32,
    num_white: i32,
}

impl Factory {
    pub fn new() -> Self {
        Factory {
            num_red: 0,
            num_green: 0,
            num_blue: 0,
            num_yellow: 0,
            num_white: 0,
        }
    }
}

#[derive(Debug)]
struct GameBoard {
    my_num_players: i32,
    max_num_factories: i32,
    tile_pool: HashMap<TileColor, i32>,
    tile_bag: Vec<TileColor>,
    factories: Vec<Factory>,
    white_tile_in_pool: bool,
    last_round: bool,
}

impl GameBoard {
    pub fn new() -> Self {
        let num_players = 2;
        let mut tile_bag:Vec<TileColor> = Vec::new();
        let gb = GameBoard {
            my_num_players: num_players,
            max_num_factories: num_players * 2 + 1,
            tile_pool: HashMap::new(),
            tile_bag: Vec::new(),
            factories: Vec::new(),
            white_tile_in_pool: false,
            last_round: false,
        }; // GameBoard
      gb.reset_board(&mut tile_bag);
      gb
    }
    fn reset_board<'a>(&self, tile_bag : &'a mut Vec<TileColor>) -> &'a Vec<TileColor> {
        tile_bag.clear();
        tile_bag.reserve(
            (TileColor::NUMCOLORS as i32 * NUM_TILES_PER_COLOR)
                .try_into()
                .unwrap(),
        );
        for ii in 0..TileColor::NUMCOLORS as usize {
            for _jj in 0..NUM_TILES_PER_COLOR {
                tile_bag.push(TileColor::from(ii));
            }
        }
        tile_bag
    } // fn reset_board
      // fn valid_factory_request -> bool
      // fn take_tiles_From_factory
      // fn take_tiles_from_pool
      // fn return_tiles_to_bag
      // fn deal_tiles
      // fn handle_request
      // fn end_of_round -> bool
} // impl GameBoard

#[derive(Debug)]
struct Player<'a> {
    my_score: i32,
    my_num_penalties_for_round: i32,
    my_took_pool_penalty_this_round: bool,
    my_grid: [[bool; TileColor::NUMCOLORS as usize]; TileColor::NUMCOLORS as usize],
    my_rows: [TileColor; TileColor::NUMCOLORS as usize],
    my_board_ref: &'a GameBoard,
    my_name: String,
}

impl<'a> Player<'a> {
    pub fn new(my_name: String, my_board_ref: &'a GameBoard) -> Self {
        Player {
            my_score: 0,
            my_num_penalties_for_round: 0,
            my_took_pool_penalty_this_round: false,
            my_grid: [[false; TileColor::NUMCOLORS as usize]; TileColor::NUMCOLORS as usize],
            my_rows: [TileColor::NOCOLOR; TileColor::NUMCOLORS as usize],
            my_board_ref,
            my_name,
        }
    }
    fn check_valid_move(&self, color: TileColor, row_idx: usize) -> Result<bool, InvalidMoveError> {
        if color == TileColor::NOCOLOR {
            return Err(InvalidMoveError::BADCOLOR);
        }
        let col_idx = (5 + color as usize - row_idx) % 5;
        if !(self.my_rows[row_idx as usize] == color
            || self.my_rows[row_idx as usize] == TileColor::NOCOLOR)
        {
            return Ok(false);
        }
        println!("col_idx: {}", col_idx);
        Ok(true)
    }
    fn score_tile(&mut self, _row_idx: usize, _col_idx: usize) -> i32 {
        self.my_score += 1;
        self.my_score
    }
    // fn take_turn
    // fn take_tiles_from_factory
    // fn take_tiles_from_pool
    // fn discard_from_factory
    // fn discard_from_pool
    // fn place_tiles
    // fn end_round
    // fn finalize_score
    // fn get_score_penalty
    // implement display trait to print?
} // impl PLayer

fn main() {
    // let num_factories = 4;
    // let mut factory_vec = Vec::new();
    let mut game_board = GameBoard::new();
    let mut player_1: Player = Player::new(String::from("Player1"), &game_board);
    println!("Show Player1: {:#?}", player_1);
}
