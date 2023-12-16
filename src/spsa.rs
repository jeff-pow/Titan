use std::{
    ops::RangeInclusive,
    sync::atomic::{AtomicI32, Ordering},
};

use crate::{search::lmr_reductions, tunable_param};

pub(crate) struct TunableParam {
    name: &'static str,
    value: AtomicI32,
    range: RangeInclusive<i32>,
    step: i32,
    callback: &'static (dyn Fn() + Sync),
}

impl TunableParam {
    pub(crate) fn val(&self) -> i32 {
        self.value.load(Ordering::Relaxed)
    }
}

tunable_param!(LMR_BASE, 100, 50, 150, 10, &lmr_reductions);
tunable_param!(LMR_DIVISOR, 200, 100, 300, 20, &lmr_reductions);

tunable_param!(INIT_ASP, 10, 5, 35, 5);
tunable_param!(ASP_DIVISOR, 10000, 5000, 25000, 5000);
tunable_param!(ASP_MIN_DEPTH, 4, 1, 9, 1);
tunable_param!(DELTA_EXPANSION, 1, 1, 6, 1);

tunable_param!(RFP_BETA_FACTOR, 70, 50, 90, 5);
tunable_param!(RFP_IMPROVING_FACTOR, 35, 35, 150, 15);
tunable_param!(RFP_DEPTH, 9, 7, 10, 1);

tunable_param!(NMP_DEPTH, 3, 3, 5, 1);
tunable_param!(NMP_BASE_R, 3, 1, 6, 1);
tunable_param!(NMP_DEPTH_DIVISOR, 3, 1, 4, 1);
tunable_param!(NMP_EVAL_DIVISOR, 200, 100, 300, 50);
tunable_param!(NMP_EVAL_MIN, 3, 1, 6, 1);

tunable_param!(LMP_DEPTH, 6, 4, 8, 1);
tunable_param!(LMP_IMP_BASE, 3, 1, 6, 1);
tunable_param!(LMP_NOT_IMP_BASE, 3, 1, 6, 1);
tunable_param!(LMP_NOT_IMP_FACTOR, 2, 1, 4, 1);

tunable_param!(QUIET_SEE, 50, 40, 80, 5);
tunable_param!(CAPT_SEE, 90, 70, 110, 5);
tunable_param!(SEE_DEPTH, 7, 5, 9, 1);

tunable_param!(LMR_MIN_MOVES, 2, 1, 3, 1);

#[macro_export]
macro_rules! tunable_param {
    ($name:ident, $value:expr, $min:expr, $max:expr, $step:expr, $callback:expr) => {
        pub static $name: TunableParam = TunableParam {
            name: stringify!($name),
            value: AtomicI32::new($value),
            range: ($min..=$max),
            step: $step,
            callback: $callback,
        };
    };

    ($name:ident, $value:expr, $min:expr, $max:expr, $step:expr) => {
        pub static $name: TunableParam = TunableParam {
            name: stringify!($name),
            value: AtomicI32::new($value),
            range: ($min..=$max),
            step: $step,
            callback: &|| (),
        };
    };
}
