#![allow(dead_code)]
use ndarray::{arr2, Array2, ArrayView2, Axis};
use rand::seq::SliceRandom;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TileColor {
    NOCOLOR,
    RED,
    BLUE,
    GREEN,
    YELLOW,
    WHITE,
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
        }
    }
    pub fn from(input: i32) -> Self {
        match input {
            0 => TileColor::RED,
            1 => TileColor::BLUE,
            2 => TileColor::GREEN,
            3 => TileColor::YELLOW,
            4 => TileColor::WHITE,
            _ => TileColor::NOCOLOR,
        }
    }
}
#[derive(Debug)] // TODO see if we can remove this
enum InvalidMoveError {
    BadColor,
    BadFactoryRequest,
    BadPoolRequest,
}

const NUM_TILES_PER_COLOR: i32 = 20;
const NUM_COLORS: i32 = 5;
const NUM_COLORS_AS_USIZE: usize = NUM_COLORS as usize;
const PENALTY_POINTS: [i32; 9] = [0, 1, 2, 3, 5, 7, 10, 13, 15];

#[derive(Debug)]
struct GameBoard {
    my_num_players: i32,
    max_num_factories: i32,
    tile_pool: HashMap<TileColor, i32>,
    tile_bag: Vec<TileColor>,
    tile_factories: Vec<HashMap<TileColor, i32>>,
    white_tile_in_pool: bool,
    last_round: bool,
}

impl GameBoard {
    pub fn new() -> Self {
        let num_players = 2;
        let mut gb = GameBoard {
            my_num_players: num_players,
            max_num_factories: num_players * 2 + 1,
            tile_pool: HashMap::new(),
            tile_bag: Vec::new(),
            tile_factories: Vec::new(),
            white_tile_in_pool: true,
            last_round: false,
        }; // GameBoard
        gb.reset_board();
        gb
    } // fn new
    fn reset_board(&mut self) {
        self.tile_factories.clear();
        self.tile_bag.clear();
        self.tile_bag
            .reserve((NUM_COLORS * NUM_TILES_PER_COLOR).try_into().unwrap());
        for ii in 0..NUM_COLORS {
            self.tile_pool
                .entry(TileColor::from(ii))
                .and_modify(|ct| *ct = 0)
                .or_insert(0);
            for _ in 0..NUM_TILES_PER_COLOR {
                self.tile_bag.push(TileColor::from(ii));
            }
        }
        self.white_tile_in_pool = true;
    } // fn reset_board
    fn valid_factory_request(&self, factory_idx: usize, tile_color: &TileColor) -> bool {
        // index is valid, key is valid, count for that key is > 0
        factory_idx < self.tile_factories.len()
            && self.tile_factories[factory_idx].contains_key(&tile_color)
            && *self.tile_factories[factory_idx]
                .get(&tile_color)
                .unwrap_or(&0)
                > 0
    } // fn valid_factory_request
    fn take_tiles_from_factory(
        &mut self,
        factory_idx: usize,
        tile_color: &TileColor,
    ) -> Result<i32, InvalidMoveError> {
        if !self.valid_factory_request(factory_idx, tile_color) {
            return Err(InvalidMoveError::BadFactoryRequest);
        }
        let mut this_factory = self.tile_factories.remove(factory_idx);
        let num_tiles = this_factory
            .remove(tile_color)
            .expect("We checked for validity, so how did this happen?");
        // move the other tiles to the pool
        for key in this_factory.keys() {
            let tile_count: i32 = *this_factory.get(&key).unwrap();
            self.tile_pool
                .entry(*key)
                .and_modify(|ct| *ct += tile_count)
                .or_insert(num_tiles);
        }
        Ok(num_tiles)
    } // fn take_tiles_from_factory
    fn take_tiles_from_pool(&mut self, tile_color: &TileColor) -> Result<i32, InvalidMoveError> {
        let num_tiles: i32 = *self.tile_pool.get(&tile_color).unwrap_or(&0);
        if num_tiles == 0 {
            return Err(InvalidMoveError::BadPoolRequest);
        }
        self.tile_pool
            .entry(*tile_color)
            .and_modify(|tile_count| *tile_count = 0);
        Ok(num_tiles)
    } // fn take_tiles_from_pool
    fn return_tiles_to_bag(&mut self, num_tiles: i32, color: &TileColor) {
        for _ in 0..num_tiles {
            self.tile_bag.push(*color);
        }
    } // fn return_tiles_to_bag
    fn deal_tiles(&mut self) {
        let num_tiles = self.tile_bag.len() as i32;
        let mut num_factories = std::cmp::min(num_tiles / 4, self.max_num_factories);
        if num_tiles < 4 * num_factories {
            num_factories = num_factories + 1;
        }
        let mut rng = rand::thread_rng();
        self.tile_bag.shuffle(&mut rng);
        for _ in 0..num_factories {
            let mut fact: HashMap<TileColor, i32> = HashMap::new();
            for _ in 0..4 {
                let drawn_tile = self
                    .tile_bag
                    .pop()
                    .expect("Somehow we pulled a bad tile from the bag??");
                fact.entry(drawn_tile)
                    .and_modify(|tile_count| *tile_count += 1)
                    .or_insert(1);
            }
            self.tile_factories.push(fact);
        }
    } // fn deal_tiles
      // fn handle_request
      // fn end_of_round -> bool
} // impl GameBoard

#[derive(Debug)]
struct Player<'a> {
    my_score: i32,
    my_num_penalties_for_round: i32,
    my_took_pool_penalty_this_round: bool,
    my_grid: Array2<bool>,
    my_rows: [(i32, TileColor); NUM_COLORS_AS_USIZE],
    my_board_ref: &'a GameBoard,
    my_name: String,
}

impl<'a> Player<'a> {
    pub fn new(my_name: String, my_board_ref: &'a GameBoard) -> Self {
        Player {
            my_score: 0,
            my_num_penalties_for_round: 0,
            my_took_pool_penalty_this_round: false,
            my_grid: arr2(&[[false; NUM_COLORS_AS_USIZE]; NUM_COLORS_AS_USIZE]),
            my_rows: [(0, TileColor::NOCOLOR); NUM_COLORS_AS_USIZE],
            my_board_ref,
            my_name,
        }
    }
    fn check_valid_move(&self, color: TileColor, row_idx: usize) -> Result<bool, InvalidMoveError> {
        // check if valid move
        //   grid doesn't already have this color on this row
        //   row is either empty or already has the same color
        if color == TileColor::NOCOLOR {
            return Err(InvalidMoveError::BadColor);
        }
        let col_idx = (5 + color as usize - row_idx) % 5;
        if self.my_grid[[row_idx, col_idx]] {
            return Ok(false); // already have that color on this row
        }
        if !(self.my_rows[row_idx as usize].1 == color
            || self.my_rows[row_idx as usize].1 == TileColor::NOCOLOR)
        {
            return Ok(false);
        }
        Ok(true)
    }
    pub fn score_tile(grid: &ArrayView2<bool>, tile_row: &usize, tile_col: &usize) -> i32 {
        println!("tile at {},{}", *tile_row, *tile_col);
        let grid_row = grid.row(*tile_row);
        let mut row_score: i32 = 0;
        let (before, after) = grid_row.split_at(Axis(0), *tile_col);
        for val in before.iter().rev() {
            // i think we're double counting here
            if !val {
                break;
            }
            row_score += 1;
        }
        for val in after.iter() {
            if !val {
                break;
            }
            row_score += 1;
        }
        let grid_col = grid.column(*tile_col);
        let (before, after) = grid_col.split_at(Axis(0), *tile_row);
        let mut col_score: i32 = 0;
        for val in before.iter().rev() {
            if !val {
                break;
            }
            col_score += 1;
        }
        for val in after.iter() {
            if !val {
                break;
            }
            col_score += 1;
        }
        if row_score == 1 || col_score == 1 {
            return std::cmp::max(row_score, col_score);
        }
        let tile_score = row_score + col_score;
        println!("row score: {}\ncol score: {}", row_score, col_score);
        //TODO not right yet
        tile_score
    }
    // fn take_turn
    fn take_tiles_from_factory(
        &mut self,
        _factory_idx: usize,
        color: TileColor,
        row_idx: usize,
    ) -> bool {
        if !self.check_valid_move(color, row_idx).unwrap() {
            return false;
        }
        true
        // TODO create request, send to board, recieve response
    }
    fn take_tiles_from_pool(&mut self, color: TileColor, row_idx: usize) -> bool {
        if !self.check_valid_move(color, row_idx).unwrap() {
            return false;
        }
        // TODO create request, send to board, recieve response
        true
    }
    // fn discard_from_factory
    // fn discard_from_pool
    fn place_tiles(&mut self, row_idx: usize, color: TileColor, num_tiles: i32) {
        self.my_rows[row_idx].0 += num_tiles;
        self.my_rows[row_idx].1 = color;
        let max_num_in_row: i32 = row_idx as i32 + 1;
        if self.my_rows[row_idx].0 > max_num_in_row {
            self.my_num_penalties_for_round += self.my_rows[row_idx].0 - max_num_in_row;
            self.my_rows[row_idx].0 = max_num_in_row;
        }
    }
    fn end_round_and_return_full_row(&mut self) -> bool {
        for (row_idx, row) in self.my_rows.iter_mut().enumerate() {
            if row.0 == (row_idx + 1).try_into().unwrap() {
                let col: usize = ((5 + TileColor::into(&row.1) - row_idx as i32) % 5)
                    .try_into()
                    .unwrap();
                // println!("row: {} col: {}", row_idx, col);
                self.my_grid[[row_idx, col]] = true;
                self.my_score += Self::score_tile(&self.my_grid.view(), &row_idx, &col);
                // TODO - request to return extra tiles (rowIdx = the number of tiles returned)
                row.0 = 0;
                row.1 = TileColor::NOCOLOR;
            }
        }
        self.my_score -= Self::get_score_penalty(&self.my_num_penalties_for_round);
        self.my_score = std::cmp::max(self.my_score, 0);
        self.my_took_pool_penalty_this_round = false;
        self.my_num_penalties_for_round = 0;
        for row_idx in 0..NUM_COLORS_AS_USIZE {
            let mut full_row: bool = true;
            for col_idx in 0..NUM_COLORS_AS_USIZE {
                if !self.my_grid[[row_idx, col_idx]] {
                    full_row = false;
                    break;
                }
            } // itr over elements in row
            if full_row {
                return true;
            }
        } // iter over rows in grid
        println!("{:#?}\nscore = {}", self.my_grid, self.my_score);
        false
    } // fn end_round_and_return_full_row

    // fn finalize_score
    fn get_score_penalty(num_penalties_for_round: &i32) -> i32 {
        let num_penalties: usize = *num_penalties_for_round as usize;
        if num_penalties >= PENALTY_POINTS.len() {
            return PENALTY_POINTS[PENALTY_POINTS.len() - 1];
        }
        PENALTY_POINTS[num_penalties]
    }
    // implement display trait to print?
} // impl PLayer

fn test_tile_score() {
    let mut arr = [[false; NUM_COLORS_AS_USIZE]; NUM_COLORS_AS_USIZE];
    arr[1][1] = true;
    arr[2][2] = true;
    arr[2][1] = true;
    arr[2][3] = true;
    arr[1][3] = true;
    let grid = arr2(&arr);
    println!("{:#?}", grid);
    let score2 = Player::score_tile(&grid.view(), &1, &3);
    println!("tile score: {}", score2);
}

fn main() {
    let mut game_board = GameBoard::new();
    game_board.deal_tiles();
    let _player_1: Player = Player::new(String::from("Player1"), &game_board);
    //     println!("Show game board: {:#?}", game_board);
    //     println!("Show player1: {:#?}", player_1);
}
