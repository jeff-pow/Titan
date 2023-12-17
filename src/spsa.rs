use std::{
    collections::HashMap,
    ops::RangeInclusive,
    sync::{
        atomic::{AtomicI32, Ordering},
        RwLock,
    },
};
pub const SPSA_TUNE: bool = true;

use lazy_static::lazy_static;

use crate::{max, search::lmr_reductions, tunable_param};

pub fn uci_print_tunable_params() {
    for e in MAP.read().unwrap().iter() {
        println!(
            "option name {} type spin default {} min {} max {}",
            e.0,
            e.1.val(),
            e.1.range.clone().min().unwrap(),
            e.1.range.clone().max().unwrap()
        );
    }
}

pub fn print_ob_tunable_params() {
    for e in MAP.read().unwrap().iter() {
        println!(
            "{}, int, {}, {}, {}, {}, 0.002",
            e.0,
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
    let binding = MAP.read().unwrap();
    let param = binding.get(name.as_str()).unwrap();
    param.value.store(value.parse().unwrap(), Ordering::SeqCst);
    (param.callback)();
}

pub(crate) struct TunableParam {
    #[allow(dead_code)]
    name: &'static str,
    value: AtomicI32,
    range: RangeInclusive<i32>,
    #[allow(dead_code)]
    step: f32,
    callback: &'static (dyn Fn() + Sync),
}

impl TunableParam {
    pub(crate) fn val(&self) -> i32 {
        self.value.load(Ordering::Relaxed)
    }
}

lazy_static! {
    static ref MAP: RwLock<HashMap<&'static str, &'static TunableParam>> = RwLock::new({
        let mut map = HashMap::new();
        map.insert("LMR_MIN_MOVES", &LMR_MIN_MOVES);
        map.insert("LMR_BASE", &LMR_BASE);
        map.insert("LMR_DIVISOR", &LMR_DIVISOR);

        map.insert("INIT_ASP", &INIT_ASP);
        map.insert("ASP_DIVISOR", &ASP_DIVISOR);
        map.insert("ASP_MIN_DEPTH", &ASP_MIN_DEPTH);
        map.insert("DELTA_EXPANSION", &DELTA_EXPANSION);

        map.insert("RFP_BETA_FACTOR", &RFP_BETA_FACTOR);
        map.insert("RFP_IMPROVING_FACTOR", &RFP_IMPROVING_FACTOR);
        map.insert("RFP_DEPTH", &RFP_DEPTH);

        map.insert("NMP_DEPTH", &NMP_DEPTH);
        map.insert("NMP_BASE_R", &NMP_BASE_R);
        map.insert("NMP_DEPTH_DIVISOR", &NMP_DEPTH_DIVISOR);
        map.insert("NMP_EVAL_DIVISOR", &NMP_EVAL_DIVISOR);
        map.insert("NMP_EVAL_MIN", &NMP_EVAL_MIN);

        map.insert("LMP_DEPTH", &LMP_DEPTH);
        map.insert("LMP_IMP_BASE", &LMP_IMP_BASE);
        map.insert("LMP_IMP_FACTOR", &LMP_IMP_FACTOR);
        map.insert("LMP_NOT_IMP_BASE", &LMP_NOT_IMP_BASE);
        map.insert("LMP_NOT_IMP_FACTOR", &LMP_NOT_IMP_FACTOR);

        map.insert("QUIET_SEE", &QUIET_SEE);
        map.insert("CAPT_SEE", &CAPT_SEE);
        map.insert("SEE_DEPTH", &SEE_DEPTH);

        map
    });
}

tunable_param!(LMR_MIN_MOVES, 2, 1, 3);
tunable_param!(LMR_BASE, 100, 50, 150, &lmr_reductions);
tunable_param!(LMR_DIVISOR, 200, 100, 300, &lmr_reductions);

tunable_param!(INIT_ASP, 10, 5, 35);
tunable_param!(ASP_DIVISOR, 10000, 5000, 25000);
tunable_param!(ASP_MIN_DEPTH, 4, 1, 9);
tunable_param!(DELTA_EXPANSION, 1, 1, 6);

tunable_param!(RFP_BETA_FACTOR, 70, 50, 90);
tunable_param!(RFP_IMPROVING_FACTOR, 35, 35, 150);
tunable_param!(RFP_DEPTH, 9, 7, 10);

tunable_param!(NMP_DEPTH, 3, 3, 5);
tunable_param!(NMP_BASE_R, 3, 1, 6);
tunable_param!(NMP_DEPTH_DIVISOR, 3, 1, 4);
tunable_param!(NMP_EVAL_DIVISOR, 200, 100, 300);
tunable_param!(NMP_EVAL_MIN, 3, 1, 6);

tunable_param!(LMP_DEPTH, 6, 4, 8);
tunable_param!(LMP_IMP_BASE, 3, 1, 6);
tunable_param!(LMP_IMP_FACTOR, 3, 1, 6);
tunable_param!(LMP_NOT_IMP_BASE, 3, 1, 6);
tunable_param!(LMP_NOT_IMP_FACTOR, 2, 1, 4);

tunable_param!(QUIET_SEE, 50, 40, 80);
tunable_param!(CAPT_SEE, 90, 70, 110);
tunable_param!(SEE_DEPTH, 7, 5, 9);

#[macro_export]
macro_rules! tunable_param {
    ($name:ident, $value:expr, $min:expr, $max:expr, $callback:expr) => {
        pub static $name: TunableParam = TunableParam {
            name: stringify!($name),
            value: AtomicI32::new($value),
            range: ($min..=$max),
            step: max!(0.5, (($max - $min) as f32) / 20.),
            callback: $callback,
        };
    };

    ($name:ident, $value:expr, $min:expr, $max:expr) => {
        pub static $name: TunableParam = TunableParam {
            name: stringify!($name),
            value: AtomicI32::new($value),
            range: ($min..=$max),
            step: max!(0.5, (($max - $min) as f32) / 20.),
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
