use std::io;

pub fn main_loop() {
    let mut buffer = String::new();
    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        match buffer.trim() {
            "uci" => {
                println!("id name Jeff's Chess Engine");
                println!("id author Jeff Powell");
                // TODO: Engine capabilities
                println!("uciok");
            },
            "isready" => println!("readyok"),
            "color" => todo!(),
            "ucinewgame" => todo!(),
            _ => println!("Non handled command: {}", buffer),
        }
    }
}
