use super::board_representation::game_state::{GameMove, GameMoveType, GameState, PieceType};
use super::evaluation;
use crate::logging::log;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::prelude::v1::Vec;

pub const STD_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KING_BASE_PATH: [&str; 15] = [
    "./KingBase/KingBase2019-A00-A39.pgn",
    "./KingBase/KingBase2019-A40-A79.pgn",
    "./KingBase/KingBase2019-A80-A99.pgn",
    "./KingBase/KingBase2019-B00-B19.pgn",
    "./KingBase/KingBase2019-B20-B49.pgn",
    "./KingBase/KingBase2019-B50-B99.pgn",
    "./KingBase/KingBase2019-C00-C19.pgn",
    "./KingBase/KingBase2019-C20-C59.pgn",
    "./KingBase/KingBase2019-C60-C99.pgn",
    "./KingBase/KingBase2019-D00-D29.pgn",
    "./KingBase/KingBase2019-D30-D69.pgn",
    "./KingBase/KingBase2019-D70-D99.pgn",
    "./KingBase/KingBase2019-E00-E19.pgn",
    "./KingBase/KingBase2019-E20-E59.pgn",
    "./KingBase/KingBase2019-E60-E99.pgn",
];

#[allow(dead_code)]
pub fn to_string_board(board: u64) -> String {
    let mut res_str: String = String::new();
    res_str.push_str("+---+---+---+---+---+---+---+---+\n");
    for rank in 0..8 {
        res_str.push_str("| ");
        for file in 0..8 {
            let idx = 8 * (7 - rank) + file;
            if ((board >> idx) & 1) != 0 {
                res_str.push_str("X");
            } else {
                res_str.push_str(" ");
            }
            res_str.push_str(" | ");
        }
        res_str.push_str("\n+---+---+---+---+---+---+---+---+\n");
    }
    res_str
}

pub fn parse_pgn_find_static_eval_mistakes() {
    for path in &KING_BASE_PATH {
        let res = File::open(path);
        let file = match res {
            Err(why) => panic!("{}", why.description()),
            Ok(file) => file,
        };
        let reader = BufReader::new(file);
        let parser = GameParser {
            pgn_parser: PGNParser { reader },
            is_opening: false,
            opening_load_untilply: 0usize,
            move_list: movegen::MoveList::new(),
        };
        for _game in parser.into_iter() {
            let last_game_state = &_game.1[_game.1.len() - 1];
            let res = _game.2;
            let eval = evaluation::eval_game_state(&last_game_state).final_eval;
            if res == 1 {
                if eval < 0 {
                    log(&format!("{} (1-0)\n", &last_game_state.to_fen()));
                }
            } else if res == 0 {
                if eval.abs() > 100 {
                    log(&format!("{} (1/2-1/2)\n", &last_game_state.to_fen()));
                }
            } else if res == -1 {
                if eval > 0 {
                    log(&format!("{} (0-1)\n", &last_game_state.to_fen()));
                }
            }
        }
    }
}

pub struct GameParser {
    pub pgn_parser: PGNParser,
    pub is_opening: bool,
    pub opening_load_untilply: usize,
    pub move_list: movegen::MoveList,
}

impl Iterator for GameParser {
    type Item = (Vec<GameMove>, Vec<GameState>, isize);
    fn next(&mut self) -> Option<(Vec<GameMove>, Vec<GameState>, isize)> {
        let next = self.pgn_parser.next();
        let mut vec_res: Vec<GameMove> = Vec::new();
        let mut vec_gs: Vec<GameState> = Vec::new();
        vec_gs.push(GameState::standard());
        match next {
            None => None,
            Some(res) => {
                let game = res.replace("\r", "").replace("\n", " ");
                if game.contains("--") || game.contains("*") || game.contains("..") {
                    //Invalid state
                    return Some((vec_res, vec_gs, -2));
                }
                //log(&format!("{}\n", game));
                let moves = game.split(" ").collect::<Vec<&str>>();
                for idx in 0..moves.len() - 2 {
                    let mut move_str = moves[idx];
                    if move_str.contains(".") {
                        move_str = move_str.rsplit(".").collect::<Vec<&str>>()[0];
                    }
                    if move_str.len() == 0 {
                        continue;
                    }
                    //println!("{} || len: {}", move_str, move_str.len());
                    let parsed_move = parse_move(
                        &vec_gs[vec_gs.len() - 1],
                        &mut String::from(move_str),
                        &mut self.move_list,
                    );
                    vec_gs.push(parsed_move.1);
                    vec_res.push(parsed_move.0);
                    if self.is_opening && vec_res.len() == self.opening_load_untilply {
                        break;
                    }
                }
                let last_elem = moves[moves.len() - 2];
                let mut score = 0;
                if last_elem == "1-0" {
                    score = 1;
                } else if last_elem == "0-1" {
                    score = -1;
                } else {
                    assert!(last_elem == "1/2-1/2");
                }
                Some((vec_res, vec_gs, score))
            }
        }
    }
}

pub fn parse_move(
    g: &GameState,
    move_str: &String,
    movelist: &mut movegen::MoveList,
) -> (GameMove, GameState) {
    let depth = 0;
    let mut my_string = move_str.clone();
    my_string = my_string
        .replace("#", "")
        .replace("+", "")
        .replace("=", "")
        .replace("x", "");
    movegen::generate_moves2(&g, false, movelist, depth);
    if my_string.contains("-") {
        //Castle
        //Kingside
        if my_string.len() == 3 {
            if g.color_to_move == 0 {
                assert_eq!(true, g.castle_white_kingside);
            } else {
                assert_eq!(true, g.castle_black_kingside);
            }
            let mut index = 0;
            while index < movelist.counter[depth] {
                let mv = movelist.move_list[depth][index].as_ref().unwrap();
                if mv.move_type == GameMoveType::Castle && mv.to as isize - mv.from as isize == 2 {
                    let res = mv.clone();
                    let state = make_move(&g, &res);
                    return (res, state);
                }
                index += 1;
            }
        } else {
            if g.color_to_move == 0 {
                assert_eq!(true, g.castle_white_queenside);
            } else {
                assert_eq!(true, g.castle_black_queenside);
            }
            let mut index = 0;
            while index < movelist.counter[depth] {
                let mv = movelist.move_list[depth][index].as_ref().unwrap();
                if mv.move_type == GameMoveType::Castle && mv.to as isize - mv.from as isize == -2 {
                    let res = mv.clone();
                    let state = make_move(&g, &res);
                    return (res, state);
                }
                index += 1;
            }
        }
    } else {
        let mut moving_piece_type = PieceType::Pawn;
        if my_string.starts_with("N") {
            moving_piece_type = PieceType::Knight;
            my_string = String::from(&my_string[1..]);
        } else if my_string.starts_with("B") {
            moving_piece_type = PieceType::Bishop;
            my_string = String::from(&my_string[1..]);
        } else if my_string.starts_with("R") {
            moving_piece_type = PieceType::Rook;
            my_string = String::from(&my_string[1..]);
        } else if my_string.starts_with("Q") {
            moving_piece_type = PieceType::Queen;
            my_string = String::from(&my_string[1..]);
        } else if my_string.starts_with("K") {
            moving_piece_type = PieceType::King;
            my_string = String::from(&my_string[1..]);
        }
        let mut is_promotion_move = false;
        let mut promotion_piece = PieceType::Queen;
        if my_string.ends_with("Q") {
            my_string = String::from(&my_string[..my_string.len() - 1]);
            is_promotion_move = true;
        } else if my_string.ends_with("R") {
            my_string = String::from(&my_string[..my_string.len() - 1]);
            is_promotion_move = true;
            promotion_piece = PieceType::Rook;
        } else if my_string.ends_with("B") {
            my_string = String::from(&my_string[..my_string.len() - 1]);
            is_promotion_move = true;
            promotion_piece = PieceType::Bishop;
        } else if my_string.ends_with("N") {
            my_string = String::from(&my_string[..my_string.len() - 1]);
            is_promotion_move = true;
            promotion_piece = PieceType::Knight;
        }
        if my_string.len() == 2 {
            let target_square =
                8 * match_rank(my_string.chars().nth(1)) + match_file(my_string.chars().nth(0));
            let mut index = 0;
            while index < movelist.counter[depth] {
                let mv = movelist.move_list[depth][index].as_ref().unwrap();
                if mv.to == target_square && mv.piece_type == moving_piece_type {
                    if !is_promotion_move
                        || is_promotion_move
                            && match &mv.move_type {
                                GameMoveType::Promotion(piece, _) => Some(piece),
                                _ => None,
                            } == Some(&promotion_piece)
                    {
                        let res = mv.clone();
                        let state = make_move(&g, &res);
                        return (res, state);
                    }
                }
                index += 1;
            }
        } else if my_string.len() == 3 {
            let target_square =
                8 * match_rank(my_string.chars().nth(2)) + match_file(my_string.chars().nth(1));
            let first = my_string.chars().nth(0);
            if is_file(first) {
                let file = match_file(first);
                let mut index = 0;
                while index < movelist.counter[depth] {
                    let mv = movelist.move_list[depth][index].as_ref().unwrap();
                    if mv.to == target_square
                        && mv.piece_type == moving_piece_type
                        && mv.from % 8 == file
                    {
                        if !is_promotion_move
                            || is_promotion_move
                                && match &mv.move_type {
                                    GameMoveType::Promotion(piece, _) => Some(piece),
                                    _ => None,
                                } == Some(&promotion_piece)
                        {
                            let res = mv.clone();
                            let state = make_move(&g, &res);
                            return (res, state);
                        }
                    }
                    index += 1;
                }
            } else {
                let rank = match_rank(first);
                let mut index = 0;
                while index < movelist.counter[depth] {
                    let mv = movelist.move_list[depth][index].as_ref().unwrap();
                    if mv.to == target_square
                        && mv.piece_type == moving_piece_type
                        && mv.from / 8 == rank
                    {
                        if !is_promotion_move
                            || is_promotion_move
                                && match &mv.move_type {
                                    GameMoveType::Promotion(piece, _) => Some(piece),
                                    _ => None,
                                } == Some(&promotion_piece)
                        {
                            let res = mv.clone();
                            let state = make_move(&g, &res);
                            return (res, state);
                        }
                    }
                    index += 1;
                }
            }
        } else if my_string.len() == 4 {
            let target_square =
                8 * match_rank(my_string.chars().nth(3)) + match_file(my_string.chars().nth(2));
            let from_square =
                8 * match_rank(my_string.chars().nth(1)) + match_file(my_string.chars().nth(0));
            let mut index = 0;
            while index < movelist.counter[depth] {
                let mv = movelist.move_list[depth][index].as_ref().unwrap();
                if mv.to == target_square
                    && mv.from == from_square
                    && (!is_promotion_move
                        || match &mv.move_type {
                            GameMoveType::Promotion(piece, _) => *piece == promotion_piece,
                            _ => false,
                        })
                {
                    let res = mv.clone();
                    let state = make_move(&g, &res);
                    return (res, state);
                }
                index += 1;
            }
        } else if my_string.len() == 5 {
        }
    }
    println!("{}", move_str);
    println!("{}", my_string);
    println!("{}", g);
    //println!("{:?}", available_moves);
    let mut index = 0;
    while index < movelist.counter[depth] {
        println!("{:?}", movelist.move_list[depth][index].as_ref().unwrap());
        index += 1;
    }
    panic!("Shouldn't get here");
}

pub fn is_file(c: Option<char>) -> bool {
    match c {
        None => panic!("Invalid!"),
        Some(character) => match character {
            'a' => true,
            'b' => true,
            'c' => true,
            'd' => true,
            'e' => true,
            'f' => true,
            'g' => true,
            'h' => true,
            _ => false,
        },
    }
}

pub fn match_file(c: Option<char>) -> usize {
    match c {
        None => panic!("Invalid!"),
        Some(character) => match character {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => panic!("Invalid rank!"),
        },
    }
}

pub fn match_rank(c: Option<char>) -> usize {
    match c {
        None => panic!("Invalid!"),
        Some(character) => match character {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => panic!("Invalid rank!"),
        },
    }
}

pub struct PGNParser {
    pub reader: BufReader<File>,
}

impl Iterator for PGNParser {
    type Item = (String);

    fn next(&mut self) -> Option<String> {
        let mut res_str = String::new();
        let mut line = String::new();
        let mut res = self.reader.read_line(&mut line);
        while match res {
            Err(_e) => false,
            Ok(_e) => true,
        } {
            if line.contains("1.") && !line.contains("[") {
                loop {
                    res_str.push_str(&line);
                    if res_str.contains("1-0")
                        || res_str.contains("0-1")
                        || res_str.contains("1/2-1/2")
                        || res_str.contains("*")
                    {
                        break;
                    }
                    line = String::new();
                    self.reader
                        .read_line(&mut line)
                        .expect("Reader had an error reading moves of game!");
                }
                break;
            }
            line = String::new();
            res = self.reader.read_line(&mut line);
            if let Err(e) = &res {
                if e.description()
                    .contains("stream did not contain valid UTF-8")
                {
                    res = Ok(1);
                }
            }
            if let Ok(e) = &res {
                if *e == 0usize {
                    break;
                }
            }
        }
        if res_str.len() != 0 {
            Some(res_str)
        } else {
            None
        }
    }
}
