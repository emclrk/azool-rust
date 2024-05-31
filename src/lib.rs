#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
use ndarray::{arr2, Array2, ArrayView2, Axis};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TileColor {
    RED,
    BLUE,
    GREEN,
    YELLOW,
    WHITE,
    NOCOLOR,
}
impl TileColor {
    pub fn into(&self) -> i32 {
        match self {
            TileColor::RED => 0,
            TileColor::BLUE => 1,
            TileColor::GREEN => 2,
            TileColor::YELLOW => 3,
            TileColor::WHITE => 4,
            TileColor::NOCOLOR => 99,
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
    BadColorError,
    BadFactoryRequestError,
    BadPoolRequestError,
    BadInputIoError(std::io::Error),
    BadInputParseError(ParseIntError),
    BadInputRowIdxError,
    UnknownError,
}

impl InvalidMoveError {
    fn from_io(err: std::io::Error) -> InvalidMoveError {
        InvalidMoveError::BadInputIoError(err)
    }
    fn from_parse(err: ParseIntError) -> InvalidMoveError {
        InvalidMoveError::BadInputParseError(err)
    }
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
            return Err(InvalidMoveError::BadFactoryRequestError);
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
            return Err(InvalidMoveError::BadPoolRequestError);
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
    fn end_of_round(&self) -> bool {
        // round ends when the pool and tile factories are empty
        for ii in 0..NUM_COLORS {
            if self.tile_pool[&TileColor::from(ii)] > 0 {
                return false;
            }
        }
        self.tile_factories.is_empty()
    } // fn end_of_round
} // impl GameBoard

#[derive(Debug)]
struct Player {
    my_score: i32,
    my_num_penalties_for_round: i32,
    my_took_pool_penalty_this_round: bool,
    my_grid: Array2<bool>,
    my_rows: [(i32, TileColor); NUM_COLORS_AS_USIZE],
    my_name: String,
}

impl<'a> Player {
    const PROMPT_DRAW_INPUT: &'a str = "What would you like to do?";
    const PROMPT_FACTORY_DRAW: &'a str = "[f] take from factory ";
    const PROMPT_POOL_DRAW: &'a str = "[p] take from pool ";
    const PROMPT_DISCARD: &'a str = "[d] discard tile(s) ";
    const PROMPT_PRINT_BOARD: &'a str = "[P] print game board";
    pub fn new(my_name: String) -> Self {
        Player {
            my_score: 0,
            my_num_penalties_for_round: 0,
            my_took_pool_penalty_this_round: false,
            my_grid: arr2(&[[false; NUM_COLORS_AS_USIZE]; NUM_COLORS_AS_USIZE]),
            my_rows: [(0, TileColor::NOCOLOR); NUM_COLORS_AS_USIZE],
            my_name,
        }
    }
    fn check_valid_move(&self, color: TileColor, row_idx: usize) -> Result<bool, InvalidMoveError> {
        // check if valid move
        //   grid doesn't already have this color on this row
        //   row is either empty or already has the same color
        if color == TileColor::NOCOLOR {
            return Err(InvalidMoveError::BadColorError);
        }
        let col_idx = get_col_idx(row_idx, color);
        if self.my_grid[[row_idx, col_idx]] {
            return Ok(false); // already have that color on this row
        }
        if !(self.my_rows[row_idx].1 == color || self.my_rows[row_idx].1 == TileColor::NOCOLOR) {
            return Ok(false);
        }
        Ok(true)
    }
    pub fn score_tile(grid: &ArrayView2<bool>, tile_row: &usize, tile_col: &usize) -> i32 {
        // println!("tile at {},{}", *tile_row, *tile_col);
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
    fn prompt_for_factory_idx(num_factories: usize) -> Result<usize, InvalidMoveError> {
        println!("Which factory? enter index [1-5]");
        let mut read_buf: String = String::new();
        std::io::stdin()
            .read_line(&mut read_buf)
            .map_err(InvalidMoveError::from_io)?;
        let idx = read_buf
            .trim()
            .parse::<usize>()
            .map_err(InvalidMoveError::from_parse)?;
        if idx > num_factories || idx < 1 {
            return Err(InvalidMoveError::BadFactoryRequestError);
        }
        Ok(idx - 1)
    } // fn prompt_for_factory_idx
    fn prompt_for_tile_color() -> Result<TileColor, InvalidMoveError> {
        println!("Which color? [r|b|g|y|w]");
        let mut read_buf = String::new();
        std::io::stdin()
            .read_line(&mut read_buf)
            .map_err(InvalidMoveError::from_io)?;
        match read_buf.chars().next() {
            Some('r') => Ok(TileColor::RED),
            Some('b') => Ok(TileColor::BLUE),
            Some('g') => Ok(TileColor::GREEN),
            Some('y') => Ok(TileColor::YELLOW),
            Some('w') => Ok(TileColor::WHITE),
            Some(_) => Err(InvalidMoveError::BadColorError),
            None => Err(InvalidMoveError::UnknownError),
        }
    }
    fn prompt_for_row_idx() -> Result<usize, InvalidMoveError> {
        println!("Which row? enter number [1-5]");
        let mut read_buf = String::new();
        std::io::stdin()
            .read_line(&mut read_buf)
            .map_err(InvalidMoveError::from_io)?;
        let row_idx = read_buf
            .trim()
            .parse::<usize>()
            .map_err(InvalidMoveError::from_parse)?;
        if row_idx < 1 || row_idx > 5 {
            return Err(InvalidMoveError::BadInputRowIdxError);
        }
        Ok(row_idx - 1)
    }
    fn take_turn(&self) {
        let mut full_input: bool = false;
        // TODO - request state info from game board
        // until then, use these fillers
        let num_factories = 4;
        let num_in_pool = 2;
        let mut write_buf = String::new();
        let mut read_buf = String::new();
        while !full_input {
            if num_factories > 0 {
                write_buf += Self::PROMPT_FACTORY_DRAW;
            }
            if num_in_pool > 0 {
                write_buf += Self::PROMPT_POOL_DRAW;
            }
            write_buf += Self::PROMPT_DISCARD;
            write_buf += Self::PROMPT_PRINT_BOARD;
            println!("{write_buf}");
            write_buf.clear();
            read_buf.clear();
            match std::io::stdin()
                .read_line(&mut read_buf)
                .map_err(InvalidMoveError::from_io)
            {
                Ok(_) => (),
                Err(error) => {
                    println!("ERROR: {:?}; try again", error);
                    continue;
                }
            }
            match read_buf.chars().next().unwrap() {
                'f' => {
                    let factory_idx = match Self::prompt_for_factory_idx(num_factories) {
                        Ok(idx) => idx,
                        Err(error) => {
                            println!("ERROR: {:?}; try again", error);
                            continue;
                        }
                    };
                    let tile_color: TileColor = match Self::prompt_for_tile_color() {
                        Ok(color) => color,
                        Err(error) => {
                            println!("ERROR: {:?}; try again", error);
                            continue;
                        }
                    };
                    let row_idx = match Self::prompt_for_row_idx() {
                        Ok(idx) => idx,
                        Err(error) => {
                            println!("ERROR: {:?}; try again", error);
                            continue;
                        }
                    };
                    println!(
                        "selected move: draw {:?} from factory {}; place on row {}",
                        tile_color,
                        factory_idx + 1,
                        row_idx + 1
                    );
                } // 'f'
                'p' => {
                    let tile_color: TileColor = match Self::prompt_for_tile_color() {
                        Ok(color) => color,
                        Err(error) => {
                            println!("ERROR: {:?}; try again", error);
                            continue;
                        }
                    };
                    println!("selected move: draw {:?} from pool", tile_color);
                } // 'p'
                'd' => {
                    let mut discard_input = String::new();
                    println!("From factory or pool? [f|p]");
                    match std::io::stdin()
                        .read_line(&mut discard_input)
                        .map_err(InvalidMoveError::from_io)
                    {
                        Ok(_) => (),
                        Err(error) => {
                            println!("INPUT ERROR: {:?} {}. Try again.", error, discard_input);
                            continue;
                        }
                    }
                    match discard_input.chars().next().unwrap() {
                        'f' => {
                            let factory_idx = match Self::prompt_for_factory_idx(num_factories) {
                                Ok(idx) => idx,
                                Err(error) => {
                                    println!("ERROR: {:?}; try again", error);
                                    continue;
                                }
                            };
                            let tile_color = match Self::prompt_for_tile_color() {
                                Ok(color) => color,
                                Err(error) => {
                                    println!("ERROR: {:?}; try again", error);
                                    continue;
                                }
                            };
                            println!(
                                "selected move: discard {:?} from factory {}",
                                tile_color,
                                factory_idx + 1
                            );
                        }
                        'p' => {
                            let tile_color = match Self::prompt_for_tile_color() {
                                Ok(color) => color,
                                Err(error) => {
                                    println!("ERROR: {:?}; try again", error);
                                    continue;
                                }
                            };
                            println!("selected move: discard {:?} from pool", tile_color);
                        }
                        _ => {
                            println!("Invalid input! Try again.");
                            continue;
                        }
                    } // discard input
                } // 'd'
                'P' => todo!(), // TODO: print board
                _ => {
                    println!("Invalid input! Try again.");
                    continue;
                }
            } // match read_buf.chars().next().unwrap()
        } // while !full_input
    } // fn take_turn
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
    fn discard_from_factory(&self) {
        todo!(); // discard_from_factory
    }
    fn discard_from_pool(&self) {
        todo!(); // discard_from_pool
    }
    fn place_tiles(&mut self, row_idx: usize, color: TileColor, num_tiles: i32) {
        self.my_rows[row_idx].0 += num_tiles;
        self.my_rows[row_idx].1 = color;
        let max_num_in_row: i32 = row_idx as i32 + 1;
        if self.my_rows[row_idx].0 > max_num_in_row {
            self.my_num_penalties_for_round += self.my_rows[row_idx].0 - max_num_in_row;
            self.my_rows[row_idx].0 = max_num_in_row;
        }
    } // fn place_tiles
    fn end_round_and_return_full_row(&mut self) -> bool {
        for (row_idx, row) in self.my_rows.iter_mut().enumerate() {
            if row.0 == (row_idx + 1).try_into().unwrap() {
                let col: usize = get_col_idx(row_idx, row.1);
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
    fn get_score_penalty(num_penalties_for_round: &i32) -> i32 {
        let num_penalties: usize = *num_penalties_for_round as usize;
        if num_penalties >= PENALTY_POINTS.len() {
            return PENALTY_POINTS[PENALTY_POINTS.len() - 1];
        }
        PENALTY_POINTS[num_penalties]
    } // fn get_score_penalty
    fn took_penalty(&self) -> bool {
        self.my_took_pool_penalty_this_round
    }
    // implement display trait to print?
    fn to_string(&self) -> String {
        format!("***************************\nPLAYER: {}\n", self.my_name)
    }
} // impl PLayer
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_string();
        Ok(())
    }
}
fn finalize_score(grid: &Array2<bool>) -> i32 {
    let mut score_bonus = 0;
    let mut num_rows = 0;
    for row in grid.rows() {
        let mut full_row: bool = true;
        for cell in row {
            if !cell {
                full_row = false;
                break;
            }
        }
        if full_row {
            num_rows += 1;
        }
    }
    score_bonus += num_rows * 2;
    let mut num_cols = 0;
    for col in grid.columns() {
        let mut full_col: bool = true;
        for cell in col {
            if !cell {
                full_col = false;
                break;
            }
        }
        if full_col {
            num_cols += 1;
        }
    }
    score_bonus += num_cols * 7;
    // TODO - 5 of a kind
    let mut num_fives = 0;
    for ii in 0..NUM_COLORS_AS_USIZE {
        let mut all_five: bool = true;
        for row_idx in 0..NUM_COLORS_AS_USIZE {
            let col_idx: usize = get_col_idx(row_idx, TileColor::from(ii as i32));
            if !grid[[row_idx, col_idx]] {
                all_five = false;
                break;
            }
        } // check each row for color ii
        if all_five {
            num_fives += 1;
        }
    } // iterate over all colors
    score_bonus += num_fives * 10;
    score_bonus
} // fn finalize_score
fn get_col_idx(row_idx: usize, color: TileColor) -> usize {
    // TODO -- make sure this is what we really want
    // used to do (5+color-row_idx) % 5
    (5 + color as usize + row_idx) % 5
}

pub fn print_player() {
    let p1 = Player::new("P1".to_string());
    println!("{:#?}", p1);
}
#[test]
fn test_tile_score() {
    let mut arr = [[false; NUM_COLORS_AS_USIZE]; NUM_COLORS_AS_USIZE];
    arr[1][1] = true;
    arr[2][2] = true;
    arr[2][1] = true;
    arr[2][3] = true;
    arr[1][3] = true;
    let grid = arr2(&arr);
    // println!("{:#?}", grid);
    let score = Player::score_tile(&grid.view(), &2, &1);
    assert_eq!(score, 5);
}
#[test]
fn test_score_bonuses() {
    let mut arr = [[false; NUM_COLORS_AS_USIZE]; NUM_COLORS_AS_USIZE];
    for idx in 0..NUM_COLORS_AS_USIZE {
        // setting row 1 and col 1
        arr[idx][1] = true;
        arr[1][idx] = true;
        // setting all blues
        let blue_col_idx = get_col_idx(idx, TileColor::BLUE);
        arr[idx][blue_col_idx] = true;
        // println!("setting [{idx},{blue_col_idx}]{}", TileColor::BLUE as usize);
    }
    // these shouldn't impact the bonus
    arr[0][0] = true;
    arr[0][3] = true;
    arr[4][4] = true;
    assert_eq!(finalize_score(&arr2(&arr)), 19);
    // removes row and column bonuses
    arr[1][1] = false;
    assert_eq!(finalize_score(&arr2(&arr)), 10);
}

pub fn run_game() {
    let mut game_board = GameBoard::new();
    let mut player_1 = Player::new(String::from("Player1"));
    let mut player_2 = Player::new(String::from("Player2"));
    let mut players: Vec<&mut Player> = vec![&mut player_1, &mut player_2]; // TODO see if we can make this less mutable
    let mut end_game: bool = false;
    let mut first_player_idx = 0;
    let num_players = players.len();
    while !end_game {
        game_board.deal_tiles();
        while !game_board.end_of_round() {
            for ii in 0..num_players {
                let idx = (ii + first_player_idx) % num_players;
                players[idx].take_turn();
            }
        } // !game_board.end_of_round()
        for (ii, player) in players.iter().enumerate() {
            if player.took_penalty() {
                first_player_idx = ii;
            }
        }
        for player in &mut players {
            if player.end_round_and_return_full_row() {
                end_game = true;
            }
        }
    } // !end_game
      // finalize scores, print results
} // fn run_game
