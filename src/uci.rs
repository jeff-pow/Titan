use crate::board::Board;
use crate::fen;
use crate::fen::build_board;
use crate::moves::generate_all_moves;
use rand::seq::SliceRandom;
use std::io;

pub fn main_loop() {
    let mut board = Board::new();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        if buffer.contains("uci") {
            println!("id name Jeff's Chess Engine");
            println!("id author Jeff Powell");
            // TODO: Engine capabilities
            println!("uciok");
        } else if buffer.contains("isready") {
            println!("readyok");
        } else if buffer.contains("ucinewgame") {
            board = Board::new();
        } else if buffer.starts_with("position") {
            let vec: Vec<&str> = buffer.split_whitespace().collect();
            if buffer.contains("fen") {
                board = build_board(vec[2]);
            }
        } else if buffer.starts_with("go") {
            let moves = generate_all_moves(&board);
            let m = moves.choose(&mut rand::thread_rng()).unwrap();
            println!("{}", m.to_lan());
        }
        match buffer.trim() {
            "uci" => {}
            "isready" => println!("readyok"),
            "color" => todo!(),
            "ucinewgame" => {
                board = Board::new();
            }
            _ => println!("Non handled command: {}", buffer),
        }
    }
}
