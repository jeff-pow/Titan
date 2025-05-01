use std::time::Instant;

use crate::board::Board;

impl Board {
    pub fn perft(&self, depth: usize) -> usize {
        let start = Instant::now();
        let count = self.non_bulk_perft::<true>(depth);
        let elapsed = start.elapsed().as_secs_f64();
        println!("{count} nodes in {elapsed} secs = {} nps", (count as f64 / elapsed) as u64);
        count
    }

    fn non_bulk_perft<const ROOT: bool>(&self, depth: usize) -> usize {
        if depth == 0 {
            return 1;
        }

        let mut total = 0;
        for m in self.pseudolegal_moves().iter() {
            if !self.is_legal(m) {
                continue;
            }

            if depth == 1 {
                total += 1;
            } else {
                let new_b = self.make_move(m);
                let count = new_b.non_bulk_perft::<false>(depth - 1);
                total += count;

                if ROOT {
                    println!("{}: {count}", m.to_san());
                }
            }
        }
        total
    }
}

#[cfg(test)]
mod movegen_tests {
    use std::thread;
    use std::{fs::File, io::BufRead, io::BufReader};

    use crate::board::Board;

    #[test]
    pub fn epd_perft() {
        let file = BufReader::new(File::open("./src/ethereal_perft.epd").expect("File not found"));
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
                        let depth = depth[1..].parse::<usize>().unwrap();
                        let nodes = nodes.parse::<usize>().unwrap();
                        eprintln!("test {test_num}: depth {depth} expected {nodes}");
                        assert_eq!(nodes, board.perft(depth), "Test number {test_num} failed");
                    }
                    eprintln!("{test_num} passed");
                });
            });
        });
    }

    #[test]
    pub fn berky_perft() {
        thread::scope(|s| {
            BERKY_PERFT.iter().enumerate().for_each(|(test_num, line)| {
                s.spawn(move || {
                    let vec = line.split(" ;").collect::<Vec<&str>>();
                    let mut iter = vec.iter();
                    let fen = iter.next().unwrap();
                    let board = Board::from_fen(fen);
                    for entry in iter {
                        println!("Fen: {fen}");
                        let (depth, nodes) = entry.split_once(' ').unwrap();
                        let depth = depth[1..].parse::<usize>().unwrap();
                        let nodes = nodes.parse::<usize>().unwrap();
                        eprintln!("test {test_num}: depth {depth} expected {nodes}");
                        assert_eq!(nodes, board.perft(depth), "Fen {fen} failed.");
                    }
                    eprintln!("{test_num} passed");
                });
            });
        });
    }

    const BERKY_PERFT: &[&str] = &[
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ;D5 4865609",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1 ;D5 4865609",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ;D5 193690690",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ;D6 11030083",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1 ;D5 15833292",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8 ;D5 89941194",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ;D5 164075551",
        "4k3/8/8/8/8/8/8/4K2R w K - 0 1 ;D5 133987",
        "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1 ;D5 145232",
        "4k2r/8/8/8/8/8/8/4K3 w k - 0 1 ;D5 47635",
        "r3k3/8/8/8/8/8/8/4K3 w q - 0 1 ;D5 52710",
        "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1 ;D5 532933",
        "r3k2r/8/8/8/8/8/8/4K3 w kq - 0 1 ;D5 118882",
        "8/8/8/8/8/8/6k1/4K2R w K - 0 1 ;D5 37735",
        "8/8/8/8/8/8/1k6/R3K3 w Q - 0 1 ;D5 80619",
        "4k2r/6K1/8/8/8/8/8/8 w k - 0 1 ;D5 10485",
        "r3k3/1K6/8/8/8/8/8/8 w q - 0 1 ;D5 20780",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1 ;D5 7594526",
        "r3k2r/8/8/8/8/8/8/1R2K2R w Kkq - 0 1 ;D5 8153719",
        "r3k2r/8/8/8/8/8/8/2R1K2R w Kkq - 0 1 ;D5 7736373",
        "r3k2r/8/8/8/8/8/8/R3K1R1 w Qkq - 0 1 ;D5 7878456",
        "1r2k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1 ;D5 8198901",
        "2r1k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1 ;D5 7710115",
        "r3k1r1/8/8/8/8/8/8/R3K2R w KQq - 0 1 ;D5 7848606",
        "4k3/8/8/8/8/8/8/4K2R b K - 0 1 ;D5 47635",
        "4k3/8/8/8/8/8/8/R3K3 b Q - 0 1 ;D5 52710",
        "4k2r/8/8/8/8/8/8/4K3 b k - 0 1 ;D5 133987",
        "r3k3/8/8/8/8/8/8/4K3 b q - 0 1 ;D5 145232",
        "4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1 ;D5 118882",
        "r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1 ;D5 532933",
        "8/8/8/8/8/8/6k1/4K2R b K - 0 1 ;D5 10485",
        "8/8/8/8/8/8/1k6/R3K3 b Q - 0 1 ;D5 20780",
        "4k2r/6K1/8/8/8/8/8/8 b k - 0 1 ;D5 37735",
        "r3k3/1K6/8/8/8/8/8/8 b q - 0 1 ;D5 80619",
        "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1 ;D5 7594526",
        "r3k2r/8/8/8/8/8/8/1R2K2R b Kkq - 0 1 ;D5 8198901",
        "r3k2r/8/8/8/8/8/8/2R1K2R b Kkq - 0 1 ;D5 7710115",
        "r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1 ;D5 7848606",
        "1r2k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1 ;D5 8153719",
        "2r1k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1 ;D5 7736373",
        "r3k1r1/8/8/8/8/8/8/R3K2R b KQq - 0 1 ;D5 7878456",
        "8/1n4N1/2k5/8/8/5K2/1N4n1/8 w - - 0 1 ;D5 570726",
        "8/1k6/8/5N2/8/4n3/8/2K5 w - - 0 1 ;D5 223507",
        "8/8/4k3/3Nn3/3nN3/4K3/8/8 w - - 0 1 ;D5 1198299",
        "K7/8/2n5/1n6/8/8/8/k6N w - - 0 1 ;D5 38348",
        "k7/8/2N5/1N6/8/8/8/K6n w - - 0 1 ;D5 92250",
        "8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1 ;D5 582642",
        "8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1 ;D5 288141",
        "8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1 ;D5 281190",
        "K7/8/2n5/1n6/8/8/8/k6N b - - 0 1 ;D5 92250",
        "k7/8/2N5/1N6/8/8/8/K6n b - - 0 1 ;D5 38348",
        "B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1 ;D5 1320507",
        "8/8/1B6/7b/7k/8/2B1b3/7K w - - 0 1 ;D5 1713368",
        "k7/B7/1B6/1B6/8/8/8/K6b w - - 0 1 ;D5 787524",
        "K7/b7/1b6/1b6/8/8/8/k6B w - - 0 1 ;D5 310862",
        "B6b/8/8/8/2K5/5k2/8/b6B b - - 0 1 ;D5 530585",
        "8/8/1B6/7b/7k/8/2B1b3/7K b - - 0 1 ;D5 1591064",
        "k7/B7/1B6/1B6/8/8/8/K6b b - - 0 1 ;D5 310862",
        "K7/b7/1b6/1b6/8/8/8/k6B b - - 0 1 ;D5 787524",
        "7k/RR6/8/8/8/8/rr6/7K w - - 0 1 ;D5 2161211",
        "R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1 ;D5 20506480",
        "7k/RR6/8/8/8/8/rr6/7K b - - 0 1 ;D5 2161211",
        "R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1 ;D5 20521342",
        "6kq/8/8/8/8/8/8/7K w - - 0 1 ;D5 14893",
        "6KQ/8/8/8/8/8/8/7k b - - 0 1 ;D5 14893",
        "K7/8/8/3Q4/4q3/8/8/7k w - - 0 1 ;D5 166741",
        "6qk/8/8/8/8/8/8/7K b - - 0 1 ;D5 105749",
        "6KQ/8/8/8/8/8/8/7k b - - 0 1 ;D5 14893",
        "K7/8/8/3Q4/4q3/8/8/7k b - - 0 1 ;D5 166741",
        "8/8/8/8/8/K7/P7/k7 w - - 0 1 ;D5 1347",
        "8/8/8/8/8/7K/7P/7k w - - 0 1 ;D5 1347",
        "K7/p7/k7/8/8/8/8/8 w - - 0 1 ;D5 342",
        "7K/7p/7k/8/8/8/8/8 w - - 0 1 ;D5 342",
        "8/2k1p3/3pP3/3P2K1/8/8/8/8 w - - 0 1 ;D5 7028",
        "8/8/8/8/8/K7/P7/k7 b - - 0 1 ;D5 342",
        "8/8/8/8/8/7K/7P/7k b - - 0 1 ;D5 342",
        "K7/p7/k7/8/8/8/8/8 b - - 0 1 ;D5 1347",
        "7K/7p/7k/8/8/8/8/8 b - - 0 1 ;D5 1347",
        "8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1 ;D5 5408",
        "8/8/8/8/8/4k3/4P3/4K3 w - - 0 1 ;D5 1814",
        "4k3/4p3/4K3/8/8/8/8/8 b - - 0 1 ;D5 1814",
        "8/8/7k/7p/7P/7K/8/8 w - - 0 1 ;D5 1969",
        "8/8/k7/p7/P7/K7/8/8 w - - 0 1 ;D5 1969",
        "8/8/3k4/3p4/3P4/3K4/8/8 w - - 0 1 ;D5 8296",
        "8/3k4/3p4/8/3P4/3K4/8/8 w - - 0 1 ;D5 23599",
        "8/8/3k4/3p4/8/3P4/3K4/8 w - - 0 1 ;D5 21637",
        "k7/8/3p4/8/3P4/8/8/7K w - - 0 1 ;D5 3450",
        "8/8/7k/7p/7P/7K/8/8 b - - 0 1 ;D5 1969",
        "8/8/k7/p7/P7/K7/8/8 b - - 0 1 ;D5 1969",
        "8/8/3k4/3p4/3P4/3K4/8/8 b - - 0 1 ;D5 8296",
        "8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1 ;D5 21637",
        "8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1 ;D5 23599",
        "k7/8/3p4/8/3P4/8/8/7K b - - 0 1 ;D5 3309",
        "7k/3p4/8/8/3P4/8/8/K7 w - - 0 1 ;D5 4661",
        "7k/8/8/3p4/8/8/3P4/K7 w - - 0 1 ;D5 4786",
        "k7/8/8/7p/6P1/8/8/K7 w - - 0 1 ;D5 6112",
        "k7/8/7p/8/8/6P1/8/K7 w - - 0 1 ;D5 4354",
        "k7/8/8/6p1/7P/8/8/K7 w - - 0 1 ;D5 6112",
        "k7/8/6p1/8/8/7P/8/K7 w - - 0 1 ;D5 4354",
        "k7/8/8/3p4/4p3/8/8/7K w - - 0 1 ;D5 3013",
        "k7/8/3p4/8/8/4P3/8/7K w - - 0 1 ;D5 4271",
        "7k/3p4/8/8/3P4/8/8/K7 b - - 0 1 ;D5 5014",
        "7k/8/8/3p4/8/8/3P4/K7 b - - 0 1 ;D5 4658",
        "k7/8/8/7p/6P1/8/8/K7 b - - 0 1 ;D5 6112",
        "k7/8/7p/8/8/6P1/8/K7 b - - 0 1 ;D5 4354",
        "k7/8/8/6p1/7P/8/8/K7 b - - 0 1 ;D5 6112",
        "k7/8/6p1/8/8/7P/8/K7 b - - 0 1 ;D5 4354",
        "k7/8/8/3p4/4p3/8/8/7K b - - 0 1 ;D5 4337",
        "k7/8/3p4/8/8/4P3/8/7K b - - 0 1 ;D5 4271",
        "7k/8/8/p7/1P6/8/8/7K w - - 0 1 ;D5 6112",
        "7k/8/8/p7/1P6/8/8/7K b - - 0 1 ;D5 6112",
        "7k/8/8/1p6/P7/8/8/7K w - - 0 1 ;D5 6112",
        "7k/8/8/1p6/P7/8/8/7K b - - 0 1 ;D5 6112",
        "7k/8/p7/8/8/1P6/8/7K w - - 0 1 ;D5 4354",
        "7k/8/p7/8/8/1P6/8/7K b - - 0 1 ;D5 4354",
        "7k/8/1p6/8/8/P7/8/7K w - - 0 1 ;D5 4354",
        "7k/8/1p6/8/8/P7/8/7K b - - 0 1 ;D5 4354",
        "k7/7p/8/8/8/8/6P1/K7 w - - 0 1 ;D5 7574",
        "k7/7p/8/8/8/8/6P1/K7 b - - 0 1 ;D5 7574",
        "k7/6p1/8/8/8/8/7P/K7 w - - 0 1 ;D5 7574",
        "k7/6p1/8/8/8/8/7P/K7 b - - 0 1 ;D5 7574",
        "8/Pk6/8/8/8/8/6Kp/8 w - - 0 1 ;D5 90606",
        "8/Pk6/8/8/8/8/6Kp/8 b - - 0 1 ;D5 90606",
        "3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1 ;D5 24122",
        "3k4/3pp3/8/8/8/8/3PP3/3K4 b - - 0 1 ;D5 24122",
        "8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1 ;D5 1533145",
        "8/PPPk4/8/8/8/8/4Kppp/8 b - - 0 1 ;D5 1533145",
        "n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1 ;D5 2193768",
        "n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1 ;D5 2193768",
        "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1 ;D5 3605103",
        "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1 ;D5 3605103",
    ];
}
