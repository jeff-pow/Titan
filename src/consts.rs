use crate::{moves::movelist::MAX_LEN, search::search::MAX_SEARCH_DEPTH};

type LmrReductions = [[i32; MAX_LEN + 1]; (MAX_SEARCH_DEPTH + 1) as usize];

pub struct Consts {
    pub lmr_min_moves: i32,
    pub lmr_base: i32,
    pub lmr_divisor: i32,
    pub lmr_depth: i32,
    pub lmr_table: LmrReductions,

    pub init_asp: i32,
    pub asp_divisor: i32,
    pub asp_min_depth: i32,
    pub delta_expansion: i32,

    pub rfp_beta_factor: i32,
    pub rfp_improving_factor: i32,
    pub rfp_depth: i32,

    pub nmp_depth: i32,
    pub nmp_base_r: i32,
    pub nmp_depth_divisor: i32,
    pub nmp_eval_divisor: i32,
    pub nmp_eval_min: i32,

    pub lmp_depth: i32,
    pub lmp_not_imp_base: i32,
    pub lmp_not_imp_factor: i32,
    pub lmp_imp_base: i32,
    pub lmp_imp_factor: i32,

    pub quiet_see: i32,
    pub capt_see: i32,
    pub see_depth: i32,

    pub ext_depth: i32,
    pub ext_tt_depth_margin: i32,
    pub ext_beta_mod: i32,
    pub dbl_ext_margin: i32,
    pub max_dbl_ext: i32,

    pub hist_depth_mod: i32,
    pub hist_min: i32,

    pub knight: i32,
    pub bishop: i32,
    pub rook: i32,
    pub queen: i32,
}

impl Consts {
    pub fn new() -> Self {
        let mut a = Self {
            lmr_min_moves: 2,
            lmr_base: 88,
            lmr_divisor: 188,
            lmr_depth: 2,
            lmr_table: [[0; MAX_LEN + 1]; MAX_SEARCH_DEPTH as usize + 1],

            init_asp: 10,
            asp_divisor: 9534,
            asp_min_depth: 4,
            delta_expansion: 1,

            rfp_beta_factor: 87,
            rfp_improving_factor: 27,
            rfp_depth: 7,

            nmp_depth: 2,
            nmp_base_r: 4,
            nmp_depth_divisor: 4,
            nmp_eval_divisor: 175,
            nmp_eval_min: 3,

            lmp_depth: 8,
            lmp_not_imp_base: 100,
            lmp_not_imp_factor: 32,
            lmp_imp_base: 244,
            lmp_imp_factor: 96,

            quiet_see: 46,
            capt_see: 100,
            see_depth: 9,

            ext_depth: 7,
            ext_tt_depth_margin: 2,
            ext_beta_mod: 108,
            dbl_ext_margin: 18,
            max_dbl_ext: 9,

            hist_depth_mod: 180,
            hist_min: 2282,

            knight: 302,
            bishop: 286,
            rook: 511,
            queen: 991,
        };
        a.init_lmr();
        a
    }

    fn init_lmr(&mut self) {
        for depth in 0..MAX_SEARCH_DEPTH + 1 {
            for moves_played in 0..MAX_LEN + 1 {
                let reduction = (self.lmr_base as f32 / 100.
                    + (depth as f32).ln() * (moves_played as f32).ln()
                        / (self.lmr_divisor as f32 / 100.)) as i32;
                self.lmr_table[depth as usize][moves_played] = reduction;
            }
        }
        self.lmr_table[0][0] = 0;
        self.lmr_table[1][0] = 0;
        self.lmr_table[0][1] = 0;
    }

    pub(crate) fn base_reduction(&self, depth: i32, moves_played: i32) -> i32 {
        self.lmr_table[depth.min(MAX_SEARCH_DEPTH) as usize][(moves_played as usize).min(MAX_LEN)]
    }
}
