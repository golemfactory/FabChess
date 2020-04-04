#[macro_use]
extern crate lazy_static;
extern crate rand;

pub mod bitboards;
pub mod board_representation;
pub mod evaluation;
pub mod logging;
pub mod misc;
pub mod move_generation;
pub mod pgn;
pub mod search;
#[cfg(tests)]
pub mod testing;
pub mod tuning;
pub mod uci;

use self::board_representation::game_state::GameState;
use self::move_generation::makemove::make_move;
use self::move_generation::movegen;
use self::search::reserved_memory::{ReservedAttackContainer, ReservedMoveList};
use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
pub const ENABLE_THREADS: bool = true;
#[cfg(target_arch = "wasm32")]
pub const ENABLE_THREADS: bool = false;

pub fn perft_div(g: &GameState, depth: usize) -> u64 {
    let mut count = 0u64;
    let mut movelist = ReservedMoveList::default();
    let mut attack_container = ReservedAttackContainer::default();
    let now = Instant::now();

    attack_container.attack_containers[depth].write_state(g);
    let _ = movegen::generate_moves(
        &g,
        false,
        &mut movelist.move_lists[depth],
        &attack_container.attack_containers[depth],
    );
    let len = movelist.move_lists[depth].move_list.len();
    for i in 0..len {
        let gmv = movelist.move_lists[depth].move_list[i];
        let next_g = make_move(&g, &gmv.0);
        let res = perft(&next_g, depth - 1, &mut movelist, &mut attack_container);
        println!("{:?}: {}", gmv.0, res);
        count += res;
    }
    println!("{}", count);
    let after = Instant::now();
    let dur = after.duration_since(now);
    let secs = dur.as_millis() as f64 / 1000.0;
    println!(
        "{}",
        &format!("Time {} ({} nps)", secs, count as f64 / secs)
    );
    count
}

pub fn perft(
    g: &GameState,
    depth: usize,
    movelist: &mut ReservedMoveList,
    attack_container: &mut ReservedAttackContainer,
) -> u64 {
    attack_container.attack_containers[depth].write_state(g);
    if depth == 1 {
        let _ = movegen::generate_moves(
            &g,
            false,
            &mut movelist.move_lists[depth],
            &attack_container.attack_containers[depth],
        );
        movelist.move_lists[depth].move_list.len() as u64
    } else {
        if depth == 0 {
            return 1;
        }
        let mut res = 0;
        let _ = movegen::generate_moves(
            &g,
            false,
            &mut movelist.move_lists[depth],
            &attack_container.attack_containers[depth],
        );
        let len = movelist.move_lists[depth].move_list.len();
        for i in 0..len {
            let mv = movelist.move_lists[depth].move_list[i].0;
            res += perft(&make_move(&g, &mv), depth - 1, movelist, attack_container);
        }
        res
    }
}
