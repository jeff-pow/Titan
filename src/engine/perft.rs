use std::time::Instant;

use crate::{
    board::board::Board,
    moves::{movegenerator::MGT, movelist::MoveList},
};

pub fn perft(board: &Board, depth: i32) -> usize {
    let start = Instant::now();
    let count = non_bulk_perft::<true>(board, depth);
    let elapsed = start.elapsed().as_secs_f64();
    println!("{count} nodes in {elapsed} secs = {} nps", count as f64 / elapsed);
    count
}

fn non_bulk_perft<const ROOT: bool>(board: &Board, depth: i32) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut total = 0;
    let mut moves = MoveList::default();
    board.generate_moves(MGT::All, &mut moves);
    for i in 0..moves.len() {
        let m = moves[i];
        let mut new_b = *board;
        if !new_b.make_move::<false>(m) {
            continue;
        }
        let count = non_bulk_perft::<false>(&new_b, depth - 1);
        if ROOT {
            println!("{}: {count}", m.to_san());
        }
        total += count;
    }
    total
}

#[cfg(test)]
mod movegen_tests {
    use std::thread;
    use std::{fs::File, io::BufRead, io::BufReader};

    use crate::board::board::Board;
    use crate::engine::perft::perft;

    #[test]
    pub fn epd_perft() {
        let file =
            BufReader::new(File::open("./src/engine/ethereal_perft.epd").expect("File not found"));
        let vec = file.lines().collect::<Vec<_>>();
        thread::scope(|s| {
            vec.iter().enumerate().for_each(|(test_num, line)| {
                s.spawn(move || {
                    let l = line.as_ref().unwrap().clone();
                    let vec = l.split(" ;").collect::<Vec<&str>>();
                    let mut iter = vec.iter();
                    let board = Board::from_fen(iter.next().unwrap());
                    for entry in iter {
                        let (depth, nodes) = entry.split_once(' ').unwrap();
                        let depth = depth[1..].parse::<i32>().unwrap();
                        let nodes = nodes.parse::<usize>().unwrap();
                        eprintln!("test {test_num}: depth {depth} expected {nodes}");
                        assert_eq!(nodes, perft(&board, depth), "Test number {test_num} failed");
                    }
                    eprintln!("{test_num} passed");
                });
            });
        });
    }
}
