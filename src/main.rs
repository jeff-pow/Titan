pub mod board;
pub mod engine;
pub mod init;
pub mod moves;
pub mod search;
pub mod types;

use crate::engine::uci::main_loop;
use crate::init::init;

fn main() {
    init();
    std::process::exit(0);
    main_loop();
}
