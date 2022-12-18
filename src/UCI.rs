use std::io;

pub fn main_loop() {
    let mut buffer = String::new();
    loop {
        println!("1: {}", buffer.as_str());
        buffer.clear();
        println!("2: {}", buffer.as_str());
        io::stdin().read_line(&mut buffer).unwrap();
        let mut string = String::from(&buffer);
        string.trim();
        println!("3: {}", string.as_str());
        match string.as_str() {
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
        println!("-------------");
    }
}
