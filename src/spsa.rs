use std::{
    ops::RangeInclusive,
    sync::atomic::{AtomicI32, Ordering},
};
pub const SPSA_TUNE: bool = true;

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
];

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
