#[macro_export]
macro_rules! tunable_params {
    ($($name:ident = $val:expr, $min:expr, $max:expr, $step:expr;)*) => {
        #[cfg(feature = "tuning")]
        use std::sync::atomic::Ordering;

        #[cfg(feature = "tuning")]
        pub fn list_params() {
            $(
                println!(
                    "option name {} type spin default {} min {} max {}",
                    stringify!($name),
                    $name(),
                    $min,
                    $max,
                );
            )*
        }

        #[cfg(feature = "tuning")]
        pub fn set_param(name: &str, val: i32) {
            match name {
                $(
                    stringify!($name) => vals::$name.store(val, Ordering::Relaxed),
                )*
                _ => println!("info error unknown option"),
            }
        }

        #[cfg(feature = "tuning")]
        pub fn print_params_ob() {
            $(
                println!(
                    "{}, int, {}.0, {}.0, {}.0, {}, 0.002",
                    stringify!($name),
                    $name(),
                    $min,
                    $max,
                    $step,
                );
            )*
        }

        #[cfg(feature = "tuning")]
        mod vals {
            use std::sync::atomic::AtomicI32;
            $(
            #[allow(non_upper_case_globals)]
            pub static $name: AtomicI32 = AtomicI32::new($val);
            )*
        }

        $(
        #[cfg(feature = "tuning")]
        #[inline]
        pub fn $name() -> i32 {
            vals::$name.load(Ordering::Relaxed)
        }

        #[cfg(not(feature = "tuning"))]
        #[inline]
        pub fn $name() -> i32 {
            $val
        }
        )*
    };
}

tunable_params! {
    delta_init = 5, 5, 50, 10;
    delta_div = 9533, 5000, 20000, 1;
    asp_depth = 3, 1, 12, 2;
    delta_expansion = 4, 1, 12, 2;

    iir_depth = 1, 1, 12, 2;

    rfp1 = 89, 30, 120, 10;
    rfp2 = 25, 5, 50, 5;
    rfp_depth = 8, 4, 12, 2;

    nmp1 = 4, 1, 12, 2;
    nmp2 = 4, 1, 12, 2;
    nmp3 = 175, 100, 300, 20;
    nmp4 = 5, 1, 12, 2;

    picker_value = -194, -200, 100, 300;

    lmp1 = 254, 150, 350, 20;
    lmp2 = 61, 50, 200, 20;
    lmp3 = 111, 50, 200, 20;
    lmp4 = 30, 10, 150, 10;
    lmp5 = 11, 4, 12, 2;

    fp1 = 8, 5, 25, 3;
    fp2 = 214, 100, 400, 40;
    fp3 = 66, 30, 120, 5;

    see1 = 90, 0, 200, 10;
    see2 = 52, 0, 200, 10;
    see3 = 13, 2, 18, 3;

    lmr1 = 2, 1, 3, 1;
    lmr2 = 4, 2, 12, 1;
    lmr3 = 307, 100, 500, 50;
    lmr4 = 9727, 4000, 12000, 1000;

    ext1 = 7, 0, 12, 1;
    ext2 = 20, 4, 32, 4;
    ext3 = 9, 5, 20, 2;
    ext4 = 18, 2, 50, 4;
    ext5 = 233, 50, 400, 25;

    time_fraction = 65, 60, 100, 3;

    history1 = 211, 100, 300, 30;
    history2 = 2125, 1000, 4000, 150;

    tm1 = 8, 2, 20, 3;
    tm2 = 159, 50, 250, 20;
    tm3 = 143, 50, 250, 20;
    tm4 = 115, 20, 200, 20;

    lmr_base = 90, 50, 150, 10;
    lmr_div = 189, 50, 250, 10;

    knight = 307, 250, 350, 10;
    bishop = 302, 250, 350, 10;
    rook = 515, 475, 600, 15;
    queen = 1003, 900, 1100, 15;
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
