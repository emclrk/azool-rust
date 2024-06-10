#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
use json;
use json::object;
use ndarray::{arr2, Array2, ArrayView2, Axis};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::fmt;
use std::num::ParseIntError;
use strum_macros::Display;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Display)]
enum TileColor {
    RED,
    BLUE,
    GREEN,
    YELLOW,
    WHITE,
    NOCOLOR,
}
impl TileColor {
    pub fn to_integer(&self) -> i32 {
        match self {
            TileColor::RED => 0,
            TileColor::BLUE => 1,
            TileColor::GREEN => 2,
            TileColor::YELLOW => 3,
            TileColor::WHITE => 4,
            TileColor::NOCOLOR => 99,
        }
    }
    pub fn from_integer(input: i32) -> Self {
        match input {
            0 => TileColor::RED,
            1 => TileColor::BLUE,
            2 => TileColor::GREEN,
            3 => TileColor::YELLOW,
            4 => TileColor::WHITE,
            _ => TileColor::NOCOLOR,
        }
    }
    pub fn color_string(&self) -> String {
        match self {
            TileColor::RED => String::from("\x1B[1;41mR\x1B[0m"),
            TileColor::BLUE => String::from("\x1B[1;44mB\x1B[0m"),
            TileColor::GREEN => String::from("\x1B[1;42mG\x1B[0m"),
            TileColor::YELLOW => String::from("\x1B[1;30;103mY\x1B[0m"),
            TileColor::WHITE => String::from("\x1B[1;30;107mW\x1B[0m"),
            TileColor::NOCOLOR => String::from(" "),
        }
    }
    pub fn int_to_color_string(input: i32) -> String {
        TileColor::from_integer(input).color_string()
    }
    pub fn from_string(input: &str) -> Self {
        match input {
            "RED" => TileColor::RED,
            "BLUE" => TileColor::BLUE,
            "GREEN" => TileColor::GREEN,
            "YELLOW" => TileColor::YELLOW,
            "WHITE" => TileColor::WHITE,
            _ => TileColor::NOCOLOR,
        }
    }
    pub fn to_char_symbol(&self) -> &str {
        match self {
            TileColor::RED => "r",
            TileColor::BLUE => "b",
            TileColor::GREEN => "g",
            TileColor::YELLOW => "y",
            TileColor::WHITE => "w",
            TileColor::NOCOLOR => "-",
        }
    }
} // impl TileColor
#[derive(Debug)] // TODO see if we can remove this
enum InvalidMoveError {
    BadColorError,
    BadFactoryRequestError,
    BadPoolRequestError,
    BadInputIoError(std::io::Error),
    BadInputParseError(ParseIntError),
    BadInputRowIdxError,
    BadRequestError,
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
#[derive(PartialEq)]
enum AzoolRequestType {
    ReqTypeDrawFromFactory,
    ReqTypeDrawFromPool,
    ReqTypeDiscardFromFactory,
    ReqTypeDiscardFromPool,
    ReqTypeReturnToBag,
    ReqTypeGetBoard,
    ReqTypeBeginTurn,
    ReqTypeTurnFinished,
    ReqTypeInvalid,
}
impl AzoolRequestType {
    fn get_string(&self) -> String {
        match self {
            AzoolRequestType::ReqTypeDrawFromFactory => String::from("DRAW_FROM_FACTORY"),
            AzoolRequestType::ReqTypeDrawFromPool => String::from("DRAW_FROM_POOL"),
            AzoolRequestType::ReqTypeDiscardFromFactory => String::from("DISCARD_FROM_FACTORY"),
            AzoolRequestType::ReqTypeDiscardFromPool => String::from("DISCARD_FROM_POOL"),
            AzoolRequestType::ReqTypeReturnToBag => String::from("RETURN_TO_BAG"),
            AzoolRequestType::ReqTypeGetBoard => String::from("GET_BOARD"),
            AzoolRequestType::ReqTypeBeginTurn => String::from("TAKE_TURN"),
            AzoolRequestType::ReqTypeTurnFinished => String::from("TURN_FINISHED"),
            AzoolRequestType::ReqTypeInvalid => String::from("INVALID"),
        }
    }
    fn from_string(val: &str) -> Self {
        match val {
            "DRAW_FROM_FACTORY" => AzoolRequestType::ReqTypeDrawFromFactory,
            "DRAW_FROM_POOL" => AzoolRequestType::ReqTypeDrawFromPool,
            "DISCARD_FROM_FACTORY" => AzoolRequestType::ReqTypeDiscardFromFactory,
            "DISCARD_FROM_POOL" => AzoolRequestType::ReqTypeDiscardFromPool,
            "RETURN_TO_BAG" => AzoolRequestType::ReqTypeReturnToBag,
            "GET_BOARD" => AzoolRequestType::ReqTypeGetBoard,
            "TAKE_TURN" => AzoolRequestType::ReqTypeBeginTurn,
            "TURN_FINISHED" => AzoolRequestType::ReqTypeTurnFinished,
            _ => AzoolRequestType::ReqTypeInvalid,
        }
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
        gb.deal_tiles();
        gb
    } // fn new
    fn reset_board(&mut self) {
        self.tile_factories.clear();
        self.tile_bag.clear();
        self.tile_bag
            .reserve((NUM_COLORS * NUM_TILES_PER_COLOR).try_into().unwrap());
        for ii in 0..NUM_COLORS {
            self.tile_pool
                .entry(TileColor::from_integer(ii))
                .and_modify(|ct| *ct = 0)
                .or_insert(0);
            for _ in 0..NUM_TILES_PER_COLOR {
                self.tile_bag.push(TileColor::from_integer(ii));
            }
        }
        self.white_tile_in_pool = true;
    } // fn reset_board
    fn serialize_game_board(&self) -> json::JsonValue {
        let mut fact_array: json::Array = vec![];
        for fact in &self.tile_factories {
            let mut one_factory = json::JsonValue::new_object();
            for (color, count) in fact.iter() {
                let _ = one_factory.insert(&color.to_string(), *count);
            }
            let _ = fact_array.push(one_factory);
        }
        let mut pool_tree = json::JsonValue::new_object();
        let mut num_tiles_in_pool = 0;
        for (color, count) in &self.tile_pool {
            let _ = pool_tree.insert(&color.to_string(), *count);
            num_tiles_in_pool += *count;
        }
        object! {
            "req_type" : AzoolRequestType::ReqTypeGetBoard.get_string(),
            "num_factories" : fact_array.len(),
            "factories" : fact_array,
            "num_tiles_in_pool": num_tiles_in_pool,
            "pool": pool_tree,
            "end_of_round" : self.end_of_round(),
            "white_tile_in_pool" : self.white_tile_in_pool,
        }
    }
    fn process_msg(&mut self, msg: json::JsonValue) -> Result<json::JsonValue, InvalidMoveError> {
        if !msg.has_key("req_type") {
            return Err(InvalidMoveError::BadRequestError);
        }
        let mut response = msg.clone();
        let msg_type = AzoolRequestType::from_string(msg["req_type"].as_str().unwrap()); // TODO handle error
        match msg_type {
            AzoolRequestType::ReqTypeDrawFromFactory => {
                let fact_idx: usize = msg["factory_idx"].as_usize().unwrap();
                let tile_color = TileColor::from_integer(msg["tile_color"].as_i32().unwrap());
                let result = self.take_tiles_from_factory(fact_idx, &tile_color);
                match result {
                    Ok(num) => {
                        // response["status"] = "success";  // todo this was in the old version, but is it necessary?
                        let _ = response.insert("success", true);
                        let _ = response.insert("num_tiles_returned", num);
                    }
                    Err(error) => {
                        let _ = response.insert("success", false);
                        let _ = response.insert("num_tiles_returned", 0);
                        response["error_type"] = "Invalid move".into();
                    }
                }
            }
            AzoolRequestType::ReqTypeDrawFromPool => {
                let tile_color = TileColor::from_integer(msg["tile_color"].as_i32().unwrap());
                let result = self.take_tiles_from_pool(&tile_color);
                match result {
                    Ok(num) => {
                        // response["status"] = "success";  // todo this was in the old version, but is it necessary?
                        let _ = response.insert("success", true);
                        let _ = response.insert("num_tiles_returned", num);
                        if self.white_tile_in_pool {
                            let _ = response.insert("pool_penalty", true);
                            self.white_tile_in_pool = false;
                        } else {
                            let _ = response.insert("pool_penalty", false);
                        }
                    }
                    Err(error) => {
                        let _ = response.insert("success", false);
                        let _ = response.insert("num_tiles_returned", 0);
                        response["error_type"] = "Invalid move".into();
                    }
                }
            }
            AzoolRequestType::ReqTypeReturnToBag => {
                let num_tiles = msg["num_tiles_returned"].as_i32().unwrap();
                let tile_color = TileColor::from_integer(msg["tile_color"].as_i32().unwrap());
                self.return_tiles_to_bag(num_tiles, &tile_color);
                let _ = response.insert("success", true);
            }
            AzoolRequestType::ReqTypeGetBoard => {
                response = self.serialize_game_board();
                response["current_player"] = msg["current_player"].clone(); // need to keep player id
            }
            // used for the gameboard to tell the player it's their turn. doesn't make sense here
            AzoolRequestType::ReqTypeBeginTurn => return Err(InvalidMoveError::BadRequestError),
            AzoolRequestType::ReqTypeTurnFinished => todo!(), // i dont think (?) this should happen?
            AzoolRequestType::ReqTypeDiscardFromFactory => todo!(),
            AzoolRequestType::ReqTypeDiscardFromPool => todo!(),
            AzoolRequestType::ReqTypeInvalid => return Err(InvalidMoveError::BadRequestError),
        } // match msg_type
        Ok(response)
    }
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
            if self.tile_pool[&TileColor::from_integer(ii)] > 0 {
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
    my_player_id: u8,
    my_tx_to_gb: mpsc::Sender<json::JsonValue>,
    my_rx_from_gb: mpsc::Receiver<json::JsonValue>,
}

impl<'a> Player {
    const PROMPT_DRAW_INPUT: &'a str = "What would you like to do?";
    const PROMPT_FACTORY_DRAW: &'a str = "[f] take from factory ";
    const PROMPT_POOL_DRAW: &'a str = "[p] take from pool ";
    const PROMPT_DISCARD: &'a str = "[d] discard tile(s) ";
    const PROMPT_PRINT_BOARD: &'a str = "[P] print game board";
    pub fn new(
        my_player_id: u8,
        my_tx_to_gb: mpsc::Sender<json::JsonValue>,
        my_rx_from_gb: mpsc::Receiver<json::JsonValue>,
    ) -> Self {
        Player {
            my_score: 0,
            my_num_penalties_for_round: 0,
            my_took_pool_penalty_this_round: false,
            my_grid: arr2(&[[false; NUM_COLORS_AS_USIZE]; NUM_COLORS_AS_USIZE]),
            my_rows: [(0, TileColor::NOCOLOR); NUM_COLORS_AS_USIZE],
            my_player_id,
            my_tx_to_gb,
            my_rx_from_gb,
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
    fn print_board(&self, gb_msg: json::JsonValue) {
        /*
        object! {
            "req_type" : AzoolRequestType::ReqTypeGetBoard.get_string(),
            "num_factories" : fact_array.len(),
            "factories" : fact_array,
            "num_tiles_in_pool": num_tiles_in_pool,
            "pool": pool_tree,
            "end_of_round" : self.end_of_round(),
            "white_tile_in_pool" : self.white_tile_in_pool,
        }
        */
        // TODO - some sort of text stream?
        println!("---------------------------");
        println!("Factories:");
        let mut counter = 1;
        let mut lines = String::new();
        for factory in gb_msg["factories"].members() {
            lines.push_str(format! {"{}) ", counter}.as_str());
            for (color_str, num) in factory.entries() {
                for _ in 0..num.as_usize().unwrap() {
                    lines.push_str(
                        format! {"{} ", TileColor::from_string(color_str).color_string()}.as_str(),
                    );
                }
            }
            lines.push_str("\n");
            counter += 1;
        } // iter over factories
        lines.push_str("\nPOOL:\n");
        if gb_msg["white_tile_in_pool"].as_bool().unwrap() {
            lines.push_str("[-1]\n");
        }
        for (color_str, num) in gb_msg["pool"].entries() {
            lines.push_str(format!{"{} x {}\n", TileColor::from_string(color_str).color_string(), num.as_i32().unwrap()}.as_str());
        }
        for ii in 0..NUM_COLORS_AS_USIZE {
            lines.push_str(format! {"{}) ", ii+1}.as_str());
            for jj in ii + 1..NUM_COLORS_AS_USIZE {
                lines.push_str(" ");
            }
            for jj in (0..ii + 1).rev() {
                if self.my_rows[ii].1 == TileColor::NOCOLOR
                    || jj >= self.my_rows[ii].0.try_into().unwrap()
                {
                    lines.push_str("_");
                } else if jj < self.my_rows[ii].0.try_into().unwrap() {
                    lines.push_str(self.my_rows[ii].1.color_string().as_str());
                }
            }
            // print grid row
            lines.push_str("  |");
            for jj in 0..NUM_COLORS_AS_USIZE {
                let color = TileColor::from_integer(((ii + jj) % 5) as i32);
                if self.my_grid[[ii, jj]] {
                    // print colored string
                    lines.push_str(color.color_string().as_str());
                } else {
                    // print symbol only
                    lines.push_str(color.to_char_symbol());
                }
                lines.push_str("|");
            }
            lines.push_str("\n");
        } // iterate over rows
        println!("{}", lines);
        println!("PLAYER {}", self.my_player_id);
    } // fn print_board
    fn request_game_board(&self) -> json::JsonValue {
        self.my_tx_to_gb.send(object!{"req_type":AzoolRequestType::ReqTypeGetBoard.get_string(), "current_player" : self.my_player_id}).unwrap();
        let msg = self.my_rx_from_gb.recv().unwrap();
        //TODO: good for debug (?)
        if !msg.has_key("req_type") {
            panic!("Missing key req_type!")
        } else if !msg.has_key("current_player") {
            panic!("Missing key current_player!")
        } else if msg["current_player"].as_u8().unwrap() != self.my_player_id {
            panic!(
                "wrong current player! :{}",
                msg["current_player"].as_u8().unwrap()
            );
        } else if AzoolRequestType::from_string(msg["req_type"].as_str().unwrap())
            != AzoolRequestType::ReqTypeGetBoard
        {
            panic!("wrong request type! {}", msg["req_type"]);
        } else {
            msg
        }
    } // fn request_game_board
    fn take_turn(&mut self) {
        let mut full_input: bool = false;
        let game_board_state = self.request_game_board();
        let num_factories = game_board_state["num_factories"].as_usize().unwrap();
        let num_in_pool = game_board_state["num_tiles_in_pool"].as_i32().unwrap();
        self.print_board(game_board_state);
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
                    if !self.take_tiles_from_factory(factory_idx, tile_color, row_idx) {
                        continue;
                    }
                    full_input = true;
                } // 'f'
                'p' => {
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
                        "selected move: draw {:?} from pool; place on row {:?}",
                        tile_color,
                        row_idx + 1
                    );
                    if !self.take_tiles_from_pool(tile_color, row_idx) {
                        continue;
                    }
                    full_input = true;
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
                            if !self.discard_from_factory(factory_idx, tile_color) {
                                continue;
                            }
                            full_input = true;
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
                            if !self.discard_from_pool(tile_color) {
                                continue;
                            }
                            full_input = true;
                        }
                        _ => {
                            println!("Invalid input! Try again.");
                            continue;
                        }
                    } // discard input
                } // 'd'
                'P' => {
                    let ans = self.request_game_board();
                    self.print_board(ans);
                }
                _ => {
                    println!("Invalid input! Try again.");
                    continue;
                }
            } // match read_buf.chars().next().unwrap()
        } // while !full_input
        let request = object! {"req_type": AzoolRequestType::ReqTypeTurnFinished.get_string(), "current_player" : self.my_player_id};
        self.my_tx_to_gb.send(request).unwrap();
    } // fn take_turn
    fn take_tiles_from_factory(
        &mut self,
        factory_idx: usize,
        color: TileColor,
        row_idx: usize,
    ) -> bool {
        if !self.check_valid_move(color, row_idx).unwrap() {
            return false;
        }
        let request = object! {"req_type": AzoolRequestType::ReqTypeDrawFromFactory.get_string(), "tile_color": color.to_integer(),
        "factory_idx": factory_idx, "current_player" : self.my_player_id};
        self.my_tx_to_gb.send(request).unwrap();
        let msg = self.my_rx_from_gb.recv().unwrap();
        let success: bool = msg["success"].as_bool().unwrap();
        if success {
            let num_tiles: i32 = msg["num_tiles_returned"].as_i32().unwrap();
            self.place_tiles(row_idx, color, num_tiles);
        }
        success
    }
    fn take_tiles_from_pool(&mut self, color: TileColor, row_idx: usize) -> bool {
        if !self.check_valid_move(color, row_idx).unwrap() {
            return false;
        }
        let request = object! {"req_type" : AzoolRequestType::ReqTypeDrawFromPool.get_string(), "tile_color": color.to_integer(), "current_player" : self.my_player_id};
        self.my_tx_to_gb.send(request).unwrap();
        let msg = self.my_rx_from_gb.recv().unwrap();
        let success: bool = msg["success"].as_bool().unwrap();
        if success {
            let num_tiles: i32 = msg["num_tiles_returned"].as_i32().unwrap();
            self.place_tiles(row_idx, color, num_tiles);
            if msg["pool_penalty"].as_bool().unwrap() {
                self.my_took_pool_penalty_this_round = true;
            }
        }
        success
    }
    fn discard_from_factory(&mut self, factory_idx: usize, color: TileColor) -> bool {
        let request = object! {"req_type" : AzoolRequestType::ReqTypeDiscardFromFactory.get_string(), "tile_color": color.to_integer(), "factory_idx":factory_idx, "current_player" : self.my_player_id};
        self.my_tx_to_gb.send(request).unwrap();
        let msg = self.my_rx_from_gb.recv().unwrap();
        let success: bool = msg["success"].as_bool().unwrap();
        if success {
            let num_tiles: i32 = msg["num_tiles_returned"].as_i32().unwrap();
            self.my_num_penalties_for_round += num_tiles;
            let request = object! {"req_type" : AzoolRequestType::ReqTypeReturnToBag.get_string(), "tile_color":color.to_integer(), "num_tiles_returned":num_tiles, "current_player" : self.my_player_id};
            self.my_tx_to_gb.send(request).unwrap();
        }
        success
    }
    fn discard_from_pool(&mut self, color: TileColor) -> bool {
        let request = object! {"req_type" : AzoolRequestType::ReqTypeDiscardFromPool.get_string(), "tile_color": color.to_integer(), "current_player" : self.my_player_id};
        self.my_tx_to_gb.send(request).unwrap();
        let msg = self.my_rx_from_gb.recv().unwrap();
        let success: bool = msg["success"].as_bool().unwrap();
        if success {
            if msg["pool_penalty"].as_bool().unwrap() {
                self.my_num_penalties_for_round += 1;
            }
            let num_tiles: i32 = msg["num_tiles_returned"].as_i32().unwrap();
            self.my_num_penalties_for_round += num_tiles;
            let request = object! {"req_type" : AzoolRequestType::ReqTypeReturnToBag.get_string(), "tile_color":color.to_integer(), "num_tiles_returned":num_tiles, "current_player" : self.my_player_id};
            self.my_tx_to_gb.send(request).unwrap();
        }
        success
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
            if row.0 == TryInto::<i32>::try_into(row_idx + 1).unwrap() {
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
        format!(
            "***************************\nPLAYER: {}\n",
            self.my_player_id
        )
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
            let col_idx: usize = get_col_idx(row_idx, TileColor::from_integer(ii as i32));
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
    let (tx, _) = mpsc::channel();
    let (_, rx) = mpsc::channel();
    let p1 = Player::new(0, tx, rx);
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
#[test]
#[should_panic]
fn test_msg_processing_invalid_type() {
    let mut game_board = GameBoard::new();
    let request = object! {"req_type": "brglker"};
    let result = game_board.process_msg(request).unwrap();
}
#[test]
fn test_msg_processing_factory_draw() {
    let mut game_board = GameBoard::new();
    let error_msg =
        format! {"Something wrong with our tile factories? {:#?}", game_board.tile_factories};
    let num_factories = game_board.tile_factories.len();
    let color = game_board.tile_factories[0]
        .keys()
        .next()
        .expect(error_msg.as_str());
    let num_tiles = game_board.tile_factories[0][color];
    let request = object! {"req_type" : AzoolRequestType::ReqTypeDrawFromFactory.get_string(), "tile_color" : color.to_integer(), "factory_idx" : 0};
    let result = game_board.process_msg(request).unwrap();
    assert_eq!(result["num_tiles_returned"].as_i32(), Some(num_tiles));
    assert_eq!(game_board.tile_factories.len(), num_factories - 1);
    assert!(game_board.tile_pool.len() > 0 || Some(num_tiles) == Some(4));
}
#[test]
fn test_msg_processing_pool_draw() {
    let mut game_board = GameBoard::new();
    let error_msg =
        format! {"Something wrong with our tile factories? {:#?}", game_board.tile_factories};
    let num_factories = game_board.tile_factories.len();
    while game_board.tile_factories[0].keys().len() == 1 {
        // if the first factory is all of the same tile, drawing won't put anything in the pool
        // so re-deal until it works
        println!("got all of one kind - re-dealing...");
        game_board.reset_board();
        game_board.deal_tiles();
    }
    let draw_color = game_board.tile_factories[0]
        .keys()
        .next()
        .expect(error_msg.as_str());
    let _ = game_board.process_msg(object! {"req_type" : AzoolRequestType::ReqTypeDrawFromFactory.get_string(), "tile_color" : draw_color.to_integer(), "factory_idx" : 0});
    let mut draw_color = TileColor::NOCOLOR;
    let mut num_tiles = 0;
    for (color, count) in game_board.tile_pool.iter() {
        if *count > 0 {
            draw_color = *color;
            num_tiles = *count;
            break;
        }
    }
    let result = game_board.process_msg(object!{"req_type" : AzoolRequestType::ReqTypeDrawFromPool.get_string(), "tile_color" : draw_color.to_integer()}).unwrap();
    assert_eq!(result["num_tiles_returned"].as_i32(), Some(num_tiles));
}
use std::sync::mpsc;
use std::thread;
pub fn run_game() {
    let mut game_board = GameBoard::new();
    let (gameboard_to_player1_sender, player1_receiver) = mpsc::channel();
    let (gameboard_to_player2_sender, player2_receiver) = mpsc::channel();
    let (player1_to_gameboard_sender, gameboard_receiver) = mpsc::channel();
    let player2_to_gameboard_sender = player1_to_gameboard_sender.clone();
    let mut player_1 = Player::new(1, player1_to_gameboard_sender, player1_receiver);
    let mut player_2 = Player::new(2, player2_to_gameboard_sender, player2_receiver);
    let p1_handle = thread::spawn(move || {
        let mut game_over = false;
        while !game_over {
            let msg = player_1.my_rx_from_gb.recv().unwrap();
            if msg["current_player"] == player_1.my_player_id
                && AzoolRequestType::from_string(msg["req_type"].as_str().unwrap())
                    == AzoolRequestType::ReqTypeBeginTurn
            {
                player_1.take_turn();
            }
        }
    });
    let p2_handle = thread::spawn(move || {
        let mut game_over = false;
        while !game_over {
            let msg = player_2.my_rx_from_gb.recv().unwrap();
            if msg["current_player"] == player_2.my_player_id
                && AzoolRequestType::from_string(msg["req_type"].as_str().unwrap())
                    == AzoolRequestType::ReqTypeBeginTurn
            {
                player_2.take_turn();
            }
        }
    });
    // let mut players: Vec<&mut Player> = vec![&mut player_1, &mut player_2];
    // TODO see if we can make this less mutable
    let mut end_game: bool = false;
    let mut first_player_idx = 0;
    // let num_players = players.len();
    let num_players = 2;
    while !end_game {
        game_board.tile_factories.clear(); // TODO - IMPORTANT - NEED TO REMOVE THIS ONCE EVERYTHING IS WORKING OR WE WILL LOSE TILES
        game_board.deal_tiles();
        while !game_board.end_of_round() {
            gameboard_to_player1_sender.send(
                object!{"req_type" : AzoolRequestType::ReqTypeBeginTurn.get_string(), "game_over" : end_game, "current_player" : 1}
                ).unwrap();
            loop {
                let msg = gameboard_receiver.recv();
                let ans = match msg {
                    Ok(val) => {
                        if AzoolRequestType::from_string(val["req_type"].as_str().unwrap())
                            == AzoolRequestType::ReqTypeTurnFinished
                        {
                            break;
                        }
                        let response = game_board.process_msg(val).unwrap();
                        gameboard_to_player1_sender.send(response).unwrap();
                    }
                    Err(error) => continue,
                };
            }
            gameboard_to_player2_sender.send(
                object!{"req_type" : AzoolRequestType::ReqTypeBeginTurn.get_string(), "game_over" : end_game, "current_player" : 2}
                ).unwrap();
            loop {
                let msg = gameboard_receiver.recv();
                let ans = match msg {
                    Ok(val) => {
                        if AzoolRequestType::from_string(val["req_type"].as_str().unwrap())
                            == AzoolRequestType::ReqTypeTurnFinished
                        {
                            break;
                        }
                        let response = game_board.process_msg(val).unwrap();
                        gameboard_to_player2_sender.send(response).unwrap();
                    }
                    Err(error) => continue,
                };
            }
        } // !game_board.end_of_round()
          /*
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
          */
    } // !end_game
      // finalize scores, print results
} // fn run_game
