use crate::attack_boards::init_attack_boards;
use crate::magics::init_magics;
use std::sync::Once;

static INIT: Once = Once::new();

/// Function must be called before program can run. Makes the static mut variables thread safe (er)
/// both in terms of initialization and use. Running the following functions in multithreaded tests
/// without the Once wrapper led to race conditions out the wazoo
pub fn init() {
    INIT.call_once(|| {
        init_attack_boards();
        init_magics();
    });
}
