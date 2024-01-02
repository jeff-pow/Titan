#[cfg(all(not(feature = "avx512"), feature = "avx2"))]
pub(crate) mod avx2 {

    use crate::eval::nnue::{RELU_MAX, RELU_MIN};
    use crate::eval::{Block, HIDDEN_SIZE};

    const CHUNK_SIZE: usize = 16;
    /// Number of SIMD vectors contained within one hidden layer
    const REQUIRED_ITERS: usize = HIDDEN_SIZE / CHUNK_SIZE;

    #[inline]
    pub unsafe fn flatten(acc: &Block, weights: &Block) -> i32 {
        {
            let mut sum = _mm256_setzero_si256();
            for i in 0..REQUIRED_ITERS {
                let us_vector = _mm256_loadu_epi16(&acc[i * CHUNK_SIZE]);
                let crelu_result = squared_crelu(us_vector);
                let weights = _mm256_loadu_epi16(&weights[i * CHUNK_SIZE]);
                sum = _mm256_dpwssd_epi32(sum, crelu_result, weights);
            }
            hadd(sum)
        }
    }

    #[inline]
    unsafe fn hadd(sum: __m256i) -> i32 {
        let mut xmm0;
        let mut xmm1;

        // Get the lower and upper half of the register:
        xmm0 = _mm256_castsi256_si128(sum);
        xmm1 = _mm256_extracti128_si256(sum, 1);

        // Add the lower and upper half vertically:
        xmm0 = _mm_add_epi32(xmm0, xmm1);

        // Get the upper half of the result:
        xmm1 = _mm_unpackhi_epi64(xmm0, xmm0);

        // Add the lower and upper half vertically:
        xmm0 = _mm_add_epi32(xmm0, xmm1);

        // Shuffle the result so that the lower 32-bits are directly above the second-lower 32-bits:
        xmm1 = _mm_shuffle_epi32::<0b00000001>(xmm0); // 2, 3, 0, 1

        // Add the lower 32-bits to the second-lower 32-bits vertically:
        xmm0 = _mm_add_epi32(xmm0, xmm1);

        // Cast the result to the 32-bit integer type and return it:
        _mm_cvtsi128_si32(xmm0)
    }

    #[inline]
    unsafe fn clipped_relu(i: __m256i) -> __m256i {
        use std::arch::x86_64::{_mm256_max_epi16, _mm256_min_epi16};
        let min = _mm256_set1_epi16(RELU_MIN);
        let max = _mm256_set1_epi16(RELU_MAX);

        _mm256_min_epi16(_mm256_max_epi16(i, min), max)
    }

    #[inline]
    unsafe fn squared_crelu(i: __m256i) -> __m256i {
        use std::arch::x86_64::_mm256_mullo_epi16;

        let clamp = clipped_relu(i);
        _mm256_mullo_epi16(clamp, clamp)
    }

    use std::arch::x86_64::{
        __m256i, _mm256_dpwssd_epi32, _mm256_loadu_epi16, _mm256_set1_epi16, _mm256_setzero_si256,
    };
    use std::arch::x86_64::{
        _mm256_castsi256_si128, _mm256_extracti128_si256, _mm_add_epi32, _mm_cvtsi128_si32,
        _mm_shuffle_epi32, _mm_unpackhi_epi64,
    };
}

#[cfg(feature = "avx512")]
pub(crate) mod avx512 {

    use crate::eval::accumulator::Accumulator;
    use crate::eval::nnue::{RELU_MAX, RELU_MIN};
    use crate::eval::{Block, HIDDEN_SIZE};
    use crate::types::pieces::Color;

    const CHUNK_SIZE: usize = 32;
    /// Number of SIMD vectors contained within one hidden layer
    const REQUIRED_ITERS: usize = HIDDEN_SIZE / CHUNK_SIZE;

    #[inline]
    pub unsafe fn flatten(acc: &Block, weights: &Block) -> i32 {
        {
            let mut sum = _mm512_setzero_si512();
            for i in 0..REQUIRED_ITERS {
                let us_vector = _mm512_loadu_epi16(&acc[i * CHUNK_SIZE]);
                let crelu_result = squared_crelu(us_vector);
                let weights = _mm512_loadu_epi16(&weights[i * CHUNK_SIZE]);
                sum = _mm512_dpwssd_epi32(sum, crelu_result, weights);
            }
            _mm512_reduce_add_epi32(sum)
        }
    }

    #[inline]
    unsafe fn clipped_relu(i: __m512i) -> __m512i {
        let min = _mm512_set1_epi16(RELU_MIN);
        let max = _mm512_set1_epi16(RELU_MAX);

        _mm512_min_epi16(_mm512_max_epi16(i, min), max)
    }

    #[inline]
    unsafe fn squared_crelu(i: __m512i) -> __m512i {
        let clamp = clipped_relu(i);
        _mm512_mullo_epi16(clamp, clamp)
    }

    impl Accumulator {
        pub(crate) unsafe fn avx512_activate(&mut self, weights: &Block, color: Color) {
            for i in 0..REQUIRED_ITERS {
                let weights = _mm512_loadu_epi16(&weights[i * CHUNK_SIZE]);
                let acc = _mm512_loadu_epi16(&self.0[color][i * CHUNK_SIZE]);
                let updated_acc = _mm512_add_epi16(acc, weights);
                _mm512_storeu_epi16(&mut self.0[color][i * CHUNK_SIZE], updated_acc);
            }
        }

        pub(crate) unsafe fn avx512_deactivate(&mut self, weights: &Block, color: Color) {
            for i in 0..REQUIRED_ITERS {
                let weights = _mm512_loadu_epi16(&weights[i * CHUNK_SIZE]);
                let acc = _mm512_loadu_epi16(&self.0[color][i * CHUNK_SIZE]);
                let updated_acc = _mm512_sub_epi16(acc, weights);
                _mm512_storeu_epi16(&mut self.0[color][i * CHUNK_SIZE], updated_acc);
            }
        }
    }

    use std::arch::x86_64::*;
}
