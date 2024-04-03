#[cfg(all(not(feature = "avx512"), target_feature = "avx2"))]
pub(crate) mod avx2 {
    use std::arch::x86_64::*;

    use crate::eval::network::{RELU_MAX, RELU_MIN};
    use crate::eval::{Block, HIDDEN_SIZE};

    const CHUNK_SIZE: usize = 16;
    /// Number of SIMD vectors contained within one hidden layer
    const REQUIRED_ITERS: usize = HIDDEN_SIZE / CHUNK_SIZE;

    #[inline]
    pub unsafe fn flatten(acc: &Block, weights: &Block) -> i32 {
        {
            let mut sum = _mm256_setzero_si256();
            for i in 0..REQUIRED_ITERS {
                let us_vector = _mm256_load_si256(acc.as_ptr().add(i * CHUNK_SIZE).cast());
                let weights = _mm256_load_si256(weights.as_ptr().add(i * CHUNK_SIZE).cast());
                let crelu_result = clipped_relu(us_vector);
                let v = _mm256_mullo_epi16(crelu_result, weights);
                let mul = _mm256_madd_epi16(v, crelu_result);
                sum = _mm256_add_epi32(sum, mul);
            }
            hadd_i32(sum)
        }
    }

    #[inline]
    unsafe fn hadd_i32(sum: __m256i) -> i32 {
        let upper_128 = _mm256_extracti128_si256::<1>(sum);
        let lower_128 = _mm256_castsi256_si128(sum);
        let sum_128 = _mm_add_epi32(upper_128, lower_128);

        let upper_64 = _mm_unpackhi_epi64(sum_128, sum_128);
        let sum_64 = _mm_add_epi32(upper_64, sum_128);

        let upper_32 = _mm_shuffle_epi32::<0b00_00_00_01>(sum_64);
        let sum_32 = _mm_add_epi32(upper_32, sum_64);

        _mm_cvtsi128_si32(sum_32)
    }

    #[inline]
    unsafe fn clipped_relu(i: __m256i) -> __m256i {
        let min = _mm256_set1_epi16(RELU_MIN);
        let max = _mm256_set1_epi16(RELU_MAX);

        _mm256_min_epi16(_mm256_max_epi16(i, min), max)
    }
}

#[cfg(feature = "avx512")]
pub(crate) mod avx512 {

    use std::arch::x86_64::*;

    use crate::eval::accumulator::Accumulator;
    use crate::eval::network::{RELU_MAX, RELU_MIN};
    use crate::eval::{Block, HIDDEN_SIZE, NET};
    use crate::types::pieces::Color;

    const CHUNK_SIZE: usize = 32;
    /// Number of SIMD vectors contained within one hidden layer
    const REQUIRED_ITERS: usize = HIDDEN_SIZE / CHUNK_SIZE;

    pub unsafe fn flatten(acc: &Block, weights: &Block) -> i32 {
        {
            let mut sum = _mm512_setzero_si512();
            for i in 0..REQUIRED_ITERS {
                let us_vector = _mm512_load_si512(acc.as_ptr().add(i * CHUNK_SIZE).cast());
                let weights = _mm512_load_si512(weights.as_ptr().add(i * CHUNK_SIZE).cast());
                let crelu = clipped_relu(us_vector);
                let v = _mm512_mullo_epi16(crelu, weights);
                sum = _mm512_dpwssd_epi32(sum, v, crelu);
            }
            _mm512_reduce_add_epi32(sum)
        }
    }

    unsafe fn clipped_relu(i: __m512i) -> __m512i {
        let min = _mm512_set1_epi16(RELU_MIN);
        let max = _mm512_set1_epi16(RELU_MAX);

        _mm512_min_epi16(_mm512_max_epi16(i, min), max)
    }

    impl Accumulator {
        pub(crate) unsafe fn avx512_activate(&mut self, weights: &Block, color: Color) {
            for i in 0..REQUIRED_ITERS {
                let weights = _mm512_load_si512(weights.as_ptr().add(i * CHUNK_SIZE).cast());
                let acc = _mm512_load_si512(self[color].as_ptr().add(i * CHUNK_SIZE).cast());
                let updated_acc = _mm512_add_epi16(acc, weights);
                _mm512_store_si512(self[color].as_mut_ptr().add(i * CHUNK_SIZE).cast(), updated_acc);
            }
        }

        pub(crate) unsafe fn avx512_add_sub(&mut self, old: &Accumulator, a1: usize, s1: usize, side: Color) {
            let weights = &NET.feature_weights;
            for i in 0..REQUIRED_ITERS {
                let w_acc = _mm512_load_si512(old[side].as_ptr().add(i * CHUNK_SIZE).cast());
                let w_add = _mm512_load_si512(weights[a1].as_ptr().add(i * CHUNK_SIZE).cast());
                let w_sub = _mm512_load_si512(weights[s1].as_ptr().add(i * CHUNK_SIZE).cast());

                let w_updated = _mm512_add_epi16(w_acc, w_add);
                let w_updated = _mm512_sub_epi16(w_updated, w_sub);
                _mm512_store_si512(self[side].as_mut_ptr().add(i * CHUNK_SIZE).cast(), w_updated);
            }
        }

        pub(crate) unsafe fn avx512_add_sub_sub(
            &mut self,
            old: &Accumulator,
            a1: usize,
            s1: usize,
            s2: usize,
            side: Color,
        ) {
            let weights = &NET.feature_weights;
            for i in 0..REQUIRED_ITERS {
                let w_acc = _mm512_load_si512(old[side].as_ptr().add(i * CHUNK_SIZE).cast());
                let w_add = _mm512_load_si512(weights[a1].as_ptr().add(i * CHUNK_SIZE).cast());
                let w_sub1 = _mm512_load_si512(weights[s1].as_ptr().add(i * CHUNK_SIZE).cast());
                let w_sub2 = _mm512_load_si512(weights[s2].as_ptr().add(i * CHUNK_SIZE).cast());

                let w_updated = _mm512_add_epi16(w_acc, w_add);
                let w_updated = _mm512_sub_epi16(w_updated, w_sub1);
                let w_updated = _mm512_sub_epi16(w_updated, w_sub2);
                _mm512_store_si512(self[side].as_mut_ptr().add(i * CHUNK_SIZE).cast(), w_updated);
            }
        }

        #[allow(clippy::too_many_arguments)]
        pub(crate) unsafe fn avx512_add_add_sub_sub(
            &mut self,
            old: &Accumulator,
            a1: usize,
            a2: usize,
            s1: usize,
            s2: usize,
            side: Color,
        ) {
            let weights = &NET.feature_weights;
            for i in 0..REQUIRED_ITERS {
                let w_acc = _mm512_load_si512(old[side].as_ptr().add(i * CHUNK_SIZE).cast());
                let w_add1 = _mm512_load_si512(weights[a1].as_ptr().add(i * CHUNK_SIZE).cast());
                let w_add2 = _mm512_load_si512(weights[a2].as_ptr().add(i * CHUNK_SIZE).cast());
                let w_sub1 = _mm512_load_si512(weights[s1].as_ptr().add(i * CHUNK_SIZE).cast());
                let w_sub2 = _mm512_load_si512(weights[s2].as_ptr().add(i * CHUNK_SIZE).cast());

                let w_updated = _mm512_add_epi16(w_acc, w_add1);
                let w_updated = _mm512_add_epi16(w_updated, w_add2);
                let w_updated = _mm512_sub_epi16(w_updated, w_sub1);
                let w_updated = _mm512_sub_epi16(w_updated, w_sub2);
                _mm512_store_si512(self[side].as_mut_ptr().add(i * CHUNK_SIZE).cast(), w_updated);
            }
        }
    }
}
