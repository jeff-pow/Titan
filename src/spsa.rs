use std::{
    ops::RangeInclusive,
    sync::atomic::{AtomicI32, Ordering},
};
pub const SPSA_TUNE: bool = false;

use phf::{phf_map, Map};

use crate::{max, search::lmr_reductions, tunable_param};

pub fn uci_print_tunable_params() {
    for e in &ENTRIES {
        println!(
            "option name {} type spin default {} min {} max {}",
            e.0,
            e.1.val(),
            e.1.range.clone().min().unwrap(),
            e.1.range.clone().max().unwrap()
        );
    }
}

#[allow(dead_code)]
pub fn print_ob_tunable_params() {
    for e in &ENTRIES {
        println!(
            "{}, int, {}, {}, {}, {}, 0.002",
            e.1.name,
            e.1.val(),
            e.1.range.clone().min().unwrap(),
            e.1.range.clone().max().unwrap(),
            e.1.step,
        );
    }
}

pub fn parse_param(a: &[&str]) {
    let [_, _, name, _, value] = a[..5] else { todo!() };
    let name = name.to_uppercase();
    let param = ENTRIES.get(name.as_str()).unwrap();
    param.value.store(value.parse().unwrap(), Ordering::SeqCst);
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

static ENTRIES: Map<&'static str, &'static TunableParam> = phf_map! {
    "LMR_MIN_MOVES" => &LMR_MIN_MOVES,
    "LMR_BASE" => &LMR_BASE,
    "LMR_DIVISOR" => &LMR_DIVISOR,

    "INIT_ASP" => &INIT_ASP,
    "ASP_DIVISOR" => &ASP_DIVISOR,
    "ASP_MIN_DEPTH" => &ASP_MIN_DEPTH,
    "DELTA_EXPANSION" => &DELTA_EXPANSION,

    "RFP_BETA_FACTOR" => &RFP_BETA_FACTOR,
    "RFP_IMPROVING_FACTOR" => &RFP_IMPROVING_FACTOR,
    "RFP_DEPTH" => &RFP_DEPTH,

    "NMP_DEPTH" => &NMP_DEPTH,
    "NMP_BASE_R" => &NMP_BASE_R,
    "NMP_DEPTH_DIVISOR" => &NMP_DEPTH_DIVISOR,
    "NMP_EVAL_DIVISOR" => &NMP_EVAL_DIVISOR,
    "NMP_EVAL_MIN" => &NMP_EVAL_MIN,

    "LMP_DEPTH" => &LMP_DEPTH,
    "LMP_IMP_BASE" => &LMP_IMP_BASE,
    "LMP_IMP_FACTOR" => &LMP_IMP_FACTOR,
    "LMP_NOT_IMP_BASE" => &LMP_NOT_IMP_BASE,
    "LMP_NOT_IMP_FACTOR" => &LMP_NOT_IMP_FACTOR,

    "QUIET_SEE" => &QUIET_SEE,
    "CAPT_SEE" => &CAPT_SEE,
    "SEE_DEPTH" => &SEE_DEPTH,

    "EXT_DEPTH" => &EXT_DEPTH,
    "EXT_TT_DEPTH_MARGIN" => &EXT_TT_DEPTH_MARGIN,
    "EXT_BETA_MOD" => &EXT_BETA_MOD,
    // "DBL_EXT_MARGIN" => &DBL_EXT_MARGIN,
    // "MAX_DBL_EXT" => &MAX_DBL_EXT,
};

tunable_param!(LMR_MIN_MOVES, 2);
tunable_param!(LMR_BASE, 98, &lmr_reductions);
tunable_param!(LMR_DIVISOR, 198, &lmr_reductions);

tunable_param!(INIT_ASP, 11);
tunable_param!(ASP_DIVISOR, 9901);
tunable_param!(ASP_MIN_DEPTH, 4);
tunable_param!(DELTA_EXPANSION, 1);

tunable_param!(RFP_BETA_FACTOR, 70);
tunable_param!(RFP_IMPROVING_FACTOR, 35);
tunable_param!(RFP_DEPTH, 9);

tunable_param!(NMP_DEPTH, 3);
tunable_param!(NMP_BASE_R, 3);
tunable_param!(NMP_DEPTH_DIVISOR, 3);
tunable_param!(NMP_EVAL_DIVISOR, 201);
tunable_param!(NMP_EVAL_MIN, 3);

tunable_param!(LMP_DEPTH, 6);
tunable_param!(LMP_NOT_IMP_BASE, 97);
tunable_param!(LMP_NOT_IMP_FACTOR, 49);
tunable_param!(LMP_IMP_BASE, 252);
tunable_param!(LMP_IMP_FACTOR, 99);

tunable_param!(QUIET_SEE, 48);
tunable_param!(CAPT_SEE, 89);
tunable_param!(SEE_DEPTH, 7);

tunable_param!(EXT_DEPTH, 8);
tunable_param!(EXT_TT_DEPTH_MARGIN, 3);
tunable_param!(EXT_BETA_MOD, 100);
// tunable_param!(DBL_EXT_MARGIN, 18);
// tunable_param!(MAX_DBL_EXT, 8);

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
