use super::uci_engine::UCIEngine;
use crate::board_representation::game_state::{GameMove, GameMoveType, GameState, PieceType};
use crate::board_representation::game_state_attack_container::GameStateAttackContainer;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen;
use crate::search::cache::{Cache, MAX_HASH_SIZE, MIN_HASH_SIZE};
use crate::search::cache::{MAX_LOCKS, MIN_LOCKS};
use crate::search::searcher::{
    search_move, MAX_SKIP_RATIO, MAX_THREADS, MIN_SKIP_RATIO, MIN_THREADS,
};
use crate::search::timecontrol::{TimeControl, MAX_MOVE_OVERHEAD, MIN_MOVE_OVERHEAD};
use crate::search::MAX_SEARCH_DEPTH;
use crate::uci::uci_engine::UCIOptions;
use std::io;
use std::sync::{atomic::AtomicBool, atomic::AtomicI16, atomic::AtomicU64, atomic::Ordering, Arc};
use std::thread;
use std::time::Duration;
use std::u64;

pub fn parse_loop() {
    let mut history: Vec<GameState> = vec![];
    let mut us = UCIEngine::standard();
    let stop = Arc::new(AtomicBool::new(false));
    let mut cache: Option<Arc<Cache>> = None; //Needs to be in mutex because we can still clear it
    let last_score: Arc<AtomicI16> = Arc::new(AtomicI16::new(0));
    let mut movelist = movegen::MoveList::default();
    let mut attack_container = GameStateAttackContainer::default();
    let saved_time = Arc::new(AtomicU64::new(0u64));
    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        stdin.read_line(&mut line).ok().unwrap();
        let arg: Vec<&str> = line.split_whitespace().collect();
        if arg.is_empty() {
            continue;
        }
        let cmd = arg[0];
        match cmd.trim() {
            "" => continue,
            "uci" => {
                uci(&us);
            }
            "setoption" => setoption(&arg[1..], &mut cache, &mut us),

            "ucinewgame" | "newgame" => {
                newgame(&mut us);
                if cache.is_some() {
                    cache.as_ref().unwrap().clear();
                }
                saved_time.store(0, Ordering::Relaxed);
                last_score.store(0, Ordering::Relaxed);
            }
            "isready" => isready(&us, &mut cache),
            "position" => {
                history = position(&mut us, &arg[1..], &mut movelist, &mut attack_container);
            }
            "go" => {
                if cache.is_none() {
                    cache = Some(Arc::new(Cache::with_size(
                        us.options.hash_size,
                        us.options.hash_locks,
                    )));
                }
                stop.store(false, Ordering::Relaxed);
                let (tc, depth) = go(&us, &arg[1..]);
                let mut new_history = vec![];
                for gs in &history {
                    new_history.push(gs.clone());
                }
                let new_state = us.internal_state.clone();
                let cl = Arc::clone(&stop);
                let cc = Arc::clone(cache.as_ref().unwrap());
                let st = Arc::clone(&saved_time);
                let ls = Arc::clone(&last_score);
                let options = us.options;
                thread::Builder::new()
                    .stack_size(2 * 1024 * 1024)
                    .spawn(move || {
                        start_search(cl, new_state, new_history, tc, cc, depth, st, ls, options);
                    })
                    .expect("Couldn't start thread");
            }
            "stop" => {
                stop.store(true, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(5));
            }
            "quit" => {
                break;
            }
            "d" => {
                print_internal_state(&us);
            }
            "perft" => perft(&us.internal_state, &arg[1..]),
            "static" => {
                println!(
                    "cp {}",
                    crate::evaluation::eval_game_state_from_null(&us.internal_state).final_eval
                );
            }
            _ => {
                println!("Unknown command {}", line);
            }
        }
    }
}

pub fn perft(game_state: &GameState, cmd: &[&str]) {
    let depth = cmd[0].parse::<usize>().unwrap();
    crate::perft_div(&game_state, depth);
}

pub fn start_search(
    stop: Arc<AtomicBool>,
    game_state: GameState,
    history: Vec<GameState>,
    tc: TimeControl,
    cache: Arc<Cache>,
    depth: usize,
    saved_time: Arc<AtomicU64>,
    last_score: Arc<AtomicI16>,
    uci_options: UCIOptions,
) {
    let score = search_move(
        depth as i16,
        game_state,
        history,
        stop,
        cache,
        saved_time,
        last_score.load(Ordering::Relaxed),
        uci_options,
        tc,
    )
    .0;
    if let Some(score) = score {
        last_score.store(score, Ordering::Relaxed);
    }
}

pub fn print_internal_state(engine: &UCIEngine) {
    println!("{}", engine.internal_state);
}

pub fn go(engine: &UCIEngine, cmd: &[&str]) -> (TimeControl, usize) {
    let mut wtime: u64 = 0;
    let mut btime: u64 = 0;
    let mut winc: u64 = 0;
    let mut binc: u64 = 0;
    let mut depth = MAX_SEARCH_DEPTH;
    if cmd[0].to_lowercase() == "infinite" {
        return (TimeControl::Infinite, depth);
    } else if cmd[0].to_lowercase() == "depth" {
        depth = cmd[1].parse::<usize>().unwrap();
        return (TimeControl::Infinite, depth);
    }
    let mut index = 0;
    let mut movestogo: Option<usize> = None;
    while index < cmd.len() {
        match cmd[index] {
            "wtime" => {
                wtime = cmd[index + 1].parse::<u64>().unwrap();
            }
            "btime" => {
                btime = cmd[index + 1].parse::<u64>().unwrap();
            }
            "winc" => {
                winc = cmd[index + 1].parse::<u64>().unwrap();
            }
            "binc" => {
                binc = cmd[index + 1].parse::<u64>().unwrap();
            }
            "movetime" => {
                let mvtime = cmd[index + 1].parse::<u64>().unwrap();
                return (TimeControl::MoveTime(mvtime), depth);
            }
            "movestogo" => movestogo = Some(cmd[index + 1].parse::<usize>().unwrap()),
            _ => panic!("Invalid go command"),
        };
        index += 2;
    }
    if movestogo.is_none() {
        if engine.internal_state.color_to_move == 0 {
            (TimeControl::Incremental(wtime, winc), depth)
        } else {
            (TimeControl::Incremental(btime, binc), depth)
        }
    } else if let Some(mvs) = movestogo {
        if mvs == 0 {
            panic!("movestogo = 0");
        }
        if engine.internal_state.color_to_move == 0 {
            (TimeControl::Tournament(wtime, winc, mvs), depth)
        } else {
            (TimeControl::Tournament(btime, binc, mvs), depth)
        }
    } else {
        panic!("Something went wrong in go!");
    }
}

pub fn position(
    engine: &mut UCIEngine,
    cmd: &[&str],
    movelist: &mut movegen::MoveList,
    attack_container: &mut GameStateAttackContainer,
) -> Vec<GameState> {
    let mut move_index = 1;
    match cmd[0] {
        "fen" => {
            let mut fen_string = String::new();
            while move_index < cmd.len() && cmd[move_index].to_lowercase() != "moves" {
                fen_string.push_str(cmd[move_index]);
                fen_string.push_str(" ");
                move_index += 1;
            }
            engine.internal_state = GameState::from_fen(fen_string.trim_end());
        }
        "startpos" => {
            engine.internal_state = GameState::standard();
        }
        _ => {
            panic!("Illegal position cmd");
        }
    }
    let mut history: Vec<GameState> = vec![];
    history.push(engine.internal_state.clone());
    if move_index < cmd.len() && cmd[move_index].to_lowercase() == "moves" {
        move_index += 1;
        while move_index < cmd.len() {
            //Parse the move and make it
            let mv = cmd[move_index];
            let (from, to, promo) = GameMove::string_to_move(mv);
            engine.internal_state = scout_and_make_draftmove(
                from,
                to,
                promo,
                &engine.internal_state,
                movelist,
                attack_container,
            );
            history.push(engine.internal_state.clone());
            move_index += 1;
        }
    }
    history.pop();
    history
}

pub fn scout_and_make_draftmove(
    from: usize,
    to: usize,
    promo_pieces: Option<PieceType>,
    game_state: &GameState,
    movelist: &mut movegen::MoveList,
    attack_container: &mut GameStateAttackContainer,
) -> GameState {
    attack_container.write_state(game_state);
    movegen::generate_moves(&game_state, false, movelist, attack_container);
    let mut index = 0;
    while index < movelist.counter {
        let mv = movelist.move_list[index].as_ref().unwrap();
        if mv.from as usize == from && mv.to as usize == to {
            if let GameMoveType::Promotion(ps, _) = mv.move_type {
                match promo_pieces {
                    Some(piece) => {
                        if piece != ps {
                            index += 1;
                            continue;
                        }
                    }
                    None => {
                        index += 1;
                        continue;
                    }
                }
            }
            return make_move(&game_state, &mv);
        }
        index += 1;
    }
    panic!("Invalid move; not found in list!");
}

pub fn isready(us: &UCIEngine, cache: &mut Option<Arc<Cache>>) {
    if cache.is_none() {
        *cache = Some(Arc::new(Cache::with_size(
            us.options.hash_size,
            us.options.hash_locks,
        )));
    }
    println!("readyok");
}

pub fn uci(engine: &UCIEngine) {
    engine.id_command();
    println!(
        "option name Hash type spin default {} min {} max {}",
        engine.options.hash_size, MIN_HASH_SIZE, MAX_HASH_SIZE
    );
    println!("option name ClearHash type button");
    println!(
        "option name TT_locks type spin default {} min {} max {}",
        engine.options.hash_locks, MIN_LOCKS, MAX_LOCKS
    );
    println!(
        "option name Threads type spin default {} min {} max {}",
        engine.options.threads, MIN_THREADS, MAX_THREADS
    );
    println!(
        "option name MoveOverhead type spin default {} min {} max {}",
        engine.options.move_overhead, MIN_MOVE_OVERHEAD, MAX_MOVE_OVERHEAD
    );
    println!(
        "option name DebugSMPPrint type check default {}",
        engine.options.debug_print
    );
    println!(
        "option name SMPSkipRatio type spin default {} min {} max {}",
        engine.options.skip_ratio, MIN_SKIP_RATIO, MAX_SKIP_RATIO
    );
    println!("uciok");
}

pub fn setoption(cmd: &[&str], cache: &mut Option<Arc<Cache>>, us: &mut UCIEngine) {
    let mut index = 0;
    while index < cmd.len() {
        let arg = cmd[index];
        match arg.to_lowercase().as_str() {
            "hash" => {
                let num = cmd[index + 2]
                    .parse::<usize>()
                    .expect("Invalid Hash value!");
                us.options.hash_size = num;
                println!("info String Succesfully set Hash to {}", num);
                return;
            }
            "clearhash" => {
                if cache.is_some() {
                    cache.as_ref().unwrap().clear();
                }
                println!("info String Succesfully cleared hash!");
                return;
            }
            "tt_locks" => {
                let num = cmd[index + 2]
                    .parse::<usize>()
                    .expect("Invalid TT_locks value!");
                us.options.hash_locks = num;
                println!("info String Succesfully set TT_locks to {}", num);
                return;
            }
            "threads" => {
                let num = cmd[index + 2]
                    .parse::<usize>()
                    .expect("Invalid Threads value!");
                us.options.threads = num;
                println!("info String Succesfully set Threads to {}", num);
                return;
            }
            "moveoverhead" => {
                let num = cmd[index + 2]
                    .parse::<u64>()
                    .expect("Invalid MoveOverhead value!");
                us.options.move_overhead = num;
                println!("info String Succesfully set MoveOverhad to {}", num);
                return;
            }
            "debugsmpprint" => {
                let val = cmd[index + 2]
                    .parse::<bool>()
                    .expect("Invalid DebugSMPPrint value!");
                us.options.debug_print = val;
                println!("info String Succesfully set DebugSMPPrint to {}", val);
                return;
            }
            "smpskipratio" => {
                let num = cmd[index + 2]
                    .parse::<usize>()
                    .expect("Invalid SMPSkipRatio value!");
                us.options.skip_ratio = num;
                println!("info String Succesfully set SMPSkipRatio to {}", num);
                return;
            }
            _ => {
                index += 1;
            }
        }
    }
}

pub fn newgame(engine: &mut UCIEngine) {
    engine.internal_state = GameState::standard();
}
