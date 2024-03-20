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
    delta_init = 10, 5, 50, 10;
    delta_div = 9534, 5000, 20000, 1;
    asp_depth = 4, 1, 12, 2;
    delta_expansion = 3, 1, 12, 2;

    iir_depth = 4, 1, 12, 2;

    rfp1 = 87, 30, 120, 10;
    rfp2 = 27, 5, 50, 5;
    rfp_depth = 7, 4, 12, 2;

    nmp1 = 4, 1, 12, 2;
    nmp2 = 4, 1, 12, 2;
    nmp3 = 175, 100, 300, 20;
    nmp4 = 3, 1, 12, 2;

    picker_value = -100, -200, 100, 300;

    lmp1 = 244, 150, 350, 20;
    lmp2 = 96, 50, 200, 20;
    lmp3 = 100, 50, 200, 20;
    lmp4 = 32, 10, 150, 10;
    lmp5 = 8, 4, 12, 2;

    fp1 = 9, 5, 25, 3;
    fp2 = 242, 100, 400, 40;
    fp3 = 67, 30, 120, 5;

    see1 = 100, 0, 200, 10;
    see2 = 46, 0, 200, 10;
    see3 = 9, 2, 18, 3;

    lmr1 = 2, 1, 3, 1;
    lmr2 = 4, 2, 12, 1;
    lmr3 = 300, 100, 500, 50;
    lmr4 = 8192, 4000, 12000, 1000;

    ext1 = 7, 0, 12, 1;
    ext2 = 16, 4, 32, 4;
    ext3 = 10, 5, 20, 2;
    ext4 = 18, 2, 50, 4;
    ext5 = 200, 50, 400, 25;

    time_fraction = 60, 60, 100, 3;

    history1 = 180, 100, 300, 30;
    history2 = 2282, 1000, 4000, 150;

    tm1 = 8, 2, 20, 3;
    tm2 = 150, 50, 250, 20;
    tm3 = 140, 50, 250, 20;
    tm4 = 90, 20, 200, 20;

    lmr_base = 88, 50, 150, 10;
    lmr_div = 188, 50, 250, 10;

    knight = 302, 250, 350, 10;
    bishop = 286, 250, 350, 10;
    rook = 511, 475, 600, 15;
    queen = 991, 900, 1100, 15;
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
