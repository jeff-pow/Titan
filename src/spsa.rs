use std::{
    ops::RangeInclusive,
    sync::atomic::{AtomicI32, Ordering},
};
pub const SPSA_TUNE: bool = true;

use crate::{max, search::lmr_reductions, tunable_param};

pub fn uci_print_tunable_params() {
    for e in ARR {
        println!(
            "option name {} type spin default {} min {} max {}",
            e.name,
            e.val(),
            e.range.clone().min().unwrap(),
            e.range.clone().max().unwrap()
        );
    }
}

#[allow(dead_code)]
pub fn print_ob_tunable_params() {
    for e in ARR {
        println!(
            "{}, int, {}, {}, {}, {}, 0.002",
            e.name,
            e.val(),
            e.range.clone().min().unwrap(),
            e.range.clone().max().unwrap(),
            e.step,
        );
    }
}

pub fn parse_param(a: &[&str]) {
    let [_, _, name, _, value] = a[..5] else { unimplemented!() };
    let name = name.to_uppercase();
    let param = ARR.iter().find(|&x| x.name == name).unwrap();
    param.value.store(value.parse().unwrap(), Ordering::Relaxed);
    (param.callback)();
}

pub(crate) struct TunableParam {
    name: &'static str,
    value: AtomicI32,
    range: RangeInclusive<i32>,
    step: f32,
    callback: &'static (dyn Fn() + Sync),
}

impl TunableParam {
    pub(crate) fn val(&self) -> i32 {
        self.value.load(Ordering::Relaxed)
    }
}

#[rustfmt::skip]
static ARR: &[&TunableParam]= &[
    &LMR_MIN_MOVES,
    &LMR_BASE,
    &LMR_DIVISOR,
    &LMR_DEPTH,

    &INIT_ASP,
    &ASP_DIVISOR,
    &ASP_MIN_DEPTH,
    &DELTA_EXPANSION,

    &RFP_BETA_FACTOR,
    &RFP_IMPROVING_FACTOR,
    &RFP_DEPTH,

    &NMP_DEPTH,
    &NMP_BASE_R,
    &NMP_DEPTH_DIVISOR,
    &NMP_EVAL_DIVISOR,
    &NMP_EVAL_MIN,

    &LMP_DEPTH,
    &LMP_IMP_BASE,
    &LMP_IMP_FACTOR,
    &LMP_NOT_IMP_BASE,
    &LMP_NOT_IMP_FACTOR,

    &QUIET_SEE,
    &CAPT_SEE,
    &SEE_DEPTH,

    &EXT_DEPTH,
    &EXT_TT_DEPTH_MARGIN,
    &EXT_BETA_MOD,
    &DBL_EXT_MARGIN,
    &MAX_DBL_EXT,

    &HIST_DEPTH_MOD,
    &HIST_MIN,

    &KNIGHT,
    &BISHOP,
    &ROOK,
    &QUEEN,
];

tunable_param!(LMR_MIN_MOVES, 2);
tunable_param!(LMR_BASE, 88, &lmr_reductions);
tunable_param!(LMR_DIVISOR, 188, &lmr_reductions);
tunable_param!(LMR_DEPTH, 2);

tunable_param!(INIT_ASP, 10);
tunable_param!(ASP_DIVISOR, 9534);
tunable_param!(ASP_MIN_DEPTH, 4);
tunable_param!(DELTA_EXPANSION, 1);

tunable_param!(RFP_BETA_FACTOR, 87);
tunable_param!(RFP_IMPROVING_FACTOR, 27);
tunable_param!(RFP_DEPTH, 7);

// TODO: Maybe this has to be 3?
tunable_param!(NMP_DEPTH, 2);
tunable_param!(NMP_BASE_R, 4);
tunable_param!(NMP_DEPTH_DIVISOR, 4);
tunable_param!(NMP_EVAL_DIVISOR, 175);
tunable_param!(NMP_EVAL_MIN, 3);

tunable_param!(LMP_DEPTH, 8);
tunable_param!(LMP_NOT_IMP_BASE, 100);
tunable_param!(LMP_NOT_IMP_FACTOR, 32);
tunable_param!(LMP_IMP_BASE, 244);
tunable_param!(LMP_IMP_FACTOR, 96);

tunable_param!(QUIET_SEE, 46);
tunable_param!(CAPT_SEE, 100);
tunable_param!(SEE_DEPTH, 9);

tunable_param!(EXT_DEPTH, 7);
tunable_param!(EXT_TT_DEPTH_MARGIN, 2);
tunable_param!(EXT_BETA_MOD, 108);
tunable_param!(DBL_EXT_MARGIN, 18);
tunable_param!(MAX_DBL_EXT, 9);

tunable_param!(HIST_DEPTH_MOD, 180);
tunable_param!(HIST_MIN, 2282);

tunable_param!(KNIGHT, 302);
tunable_param!(BISHOP, 286);
tunable_param!(ROOK, 511);
tunable_param!(QUEEN, 991);

#[macro_export]
macro_rules! tunable_param {
    ($name:ident, $value:expr, $callback:expr) => {
        pub static $name: TunableParam = TunableParam {
            name: stringify!($name),
            value: AtomicI32::new($value),
            range: (1..=$value * 2),
            step: max!(0.5, (($value * 2 - 1) as f32) / 20.),
            callback: $callback,
        };
    };

    ($name:ident, $value:expr) => {
        pub static $name: TunableParam = TunableParam {
            name: stringify!($name),
            value: AtomicI32::new($value),
            range: (1..=$value * 2),
            step: max!(0.5, (($value * 2 - 1) as f32) / 20.),
            callback: &|| (),
        };
    };
}

#[macro_export]
macro_rules! max {
    ($a:expr, $b:expr) => {
        if $a < $b {
            $b
        } else {
            $a
        }
    };
}
