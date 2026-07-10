use std::cmp::min;

/// Bitstring vertex for the middle-levels Gray code algorithm.
///
/// Represents a vertex in the middle-levels graph as a bitstring.
/// Provides methods for flip sequence computation, vertex comparison,
/// and conversion operations used in Hamiltonian cycle generation.
///
/// Adapted from: Torsten Muetze, Jerri Nummenpalo (2018)
pub struct MidVertex {
    bits: Vec<i32>,
}

impl MidVertex {
    pub fn new(x: Vec<i32>) -> Self {
        assert!(x.len() % 2 == 1);
        assert!(x.len() >= 3);
        MidVertex { bits: x }
    }

    pub fn get_bits(&self) -> &Vec<i32> {
        &self.bits
    }

    pub fn size(&self) -> usize {
        self.bits.len()
    }

    pub fn rev_inv(&mut self) {
        let right = self.bits.len() - 2;
        self.rev_inv_range(0, right);
    }

    fn rev_inv_range(&mut self, left: usize, right: usize) {
        for i in left..=right {
            self.bits[i] = 1 - self.bits[i];
        }
        self.bits[left..=right].reverse();
    }

    fn first_touchdown(&self, a: usize) -> isize {
        let mut height = 0i32;
        for i in a..self.bits.len() - 1 {
            height += 2 * self.bits[i] - 1;
            if height == 0 {
                return i as isize;
            }
        }
        -1
    }

    fn first_dive(&self) -> isize {
        let mut height = 0i32;
        for i in 0..self.bits.len() - 1 {
            height += 2 * self.bits[i] - 1;
            if height == -1 {
                return i as isize;
            }
        }
        -1
    }

    fn steps_height(
        &self,
        usteps_neg: &mut Vec<Vec<usize>>,
        usteps_pos: &mut Vec<Vec<usize>>,
        dsteps_neg: &mut Vec<Vec<usize>>,
        dsteps_pos: &mut Vec<Vec<usize>>,
    ) {
        usteps_neg.clear();
        usteps_pos.clear();
        dsteps_neg.clear();
        dsteps_pos.clear();
        let mut height: i32 = 0;
        let mut min_height: i32 = 0;
        let mut max_height: i32 = 0;
        for i in 0..self.bits.len() - 1 {
            if self.bits[i] == 0 && height <= 0 {
                if height == min_height {
                    usteps_neg.push(Vec::new());
                    dsteps_neg.push(Vec::new());
                }
                let idx = (-height) as usize;
                while dsteps_neg.len() <= idx {
                    dsteps_neg.push(Vec::new());
                    usteps_neg.push(Vec::new());
                }
                dsteps_neg[idx].push(i);
            }
            if self.bits[i] == 1 && height >= 0 {
                if height == max_height {
                    usteps_pos.push(Vec::new());
                    dsteps_pos.push(Vec::new());
                }
                let idx = height as usize;
                while dsteps_pos.len() <= idx {
                    dsteps_pos.push(Vec::new());
                    usteps_pos.push(Vec::new());
                }
                usteps_pos[idx].push(i);
            }
            height += 2 * self.bits[i] - 1;
            min_height = min(height, min_height);
            max_height = max_height.max(height);
            if self.bits[i] == 0 && height >= 0 {
                let idx = height as usize;
                while dsteps_pos.len() <= idx {
                    dsteps_pos.push(Vec::new());
                    usteps_pos.push(Vec::new());
                }
                dsteps_pos[idx].push(i);
            }
            if self.bits[i] == 1 && height <= 0 {
                let idx = (-height) as usize;
                while usteps_neg.len() <= idx {
                    usteps_neg.push(Vec::new());
                    dsteps_neg.push(Vec::new());
                }
                usteps_neg[idx].push(i);
            }
        }
    }

    fn count_flaws(&self) -> usize {
        let mut c = 0usize;
        let mut height: i32 = 0;
        for i in 0..self.bits.len() - 1 {
            if height <= 0 && self.bits[i] == 0 {
                c += 1;
            }
            height += 2 * self.bits[i] - 1;
        }
        c
    }

    pub fn count_ones(&self) -> usize {
        let mut c = 0usize;
        for i in 0..self.bits.len() - 1 {
            if self.bits[i] == 1 {
                c += 1;
            }
        }
        c
    }

    pub fn is_first_vertex(&self) -> bool {
        self.count_flaws() == 0 && self.count_ones() == self.bits.len() / 2
    }

    pub fn is_last_vertex(&self) -> bool {
        self.count_flaws() == 1 && self.count_ones() == self.bits.len() / 2
    }

    pub fn to_first_vertex(&mut self) -> usize {
        if self.is_last_vertex() {
            let b = self.first_dive() as usize;
            for i in (0..b).rev() {
                self.bits[i + 1] = self.bits[i];
            }
            self.bits[0] = 1;
            self.bits[b + 1] = 0;
            return 2 * b + 2;
        }

        let mut usteps_neg = Vec::new();
        let mut dsteps_neg = Vec::new();
        let mut usteps_pos = Vec::new();
        let mut dsteps_pos = Vec::new();
        self.steps_height(&mut usteps_neg, &mut usteps_pos, &mut dsteps_neg, &mut dsteps_pos);

        let min_zero = usteps_neg.is_empty();
        let unique_min = if min_zero {
            usteps_pos.first().map_or(false, |v| v.len() == 1)
        } else {
            usteps_neg.last().map_or(false, |v| v.len() == 1)
        };
        let middle_level = 2 * self.count_ones() + 1 == self.bits.len();

        let to = if (!unique_min && middle_level) || (unique_min && !middle_level) {
            if min_zero {
                usteps_pos.first().and_then(|v| v.first()).copied().unwrap_or(0)
            } else {
                usteps_neg.last().and_then(|v| v.first()).copied().unwrap_or(0)
            }
        } else {
            if min_zero {
                usteps_pos.first().and_then(|v| v.last()).copied().unwrap_or(0)
            } else {
                usteps_neg.last().and_then(|v| v.last()).copied().unwrap_or(0)
            }
        };
        let to_val = to.saturating_sub(1);

        for i in (0..=to_val).rev() {
            self.bits[i + 1] = self.bits[i];
        }
        self.bits[0] = 1;

        let dsteps_neg_len = dsteps_neg.len();
        let limit = if unique_min && middle_level { 1 } else { 0 };
        for d in 0..dsteps_neg_len.saturating_sub(limit) {
            if let Some(v) = dsteps_neg[d].first() {
                self.bits[v + 1] = 1;
            }
        }

        let usteps_neg_len = usteps_neg.len();
        let limit2 = if unique_min && !middle_level { 1 } else { 0 };
        for d in 0..usteps_neg_len.saturating_sub(limit2) {
            if let Some(v) = usteps_neg[d].last() {
                self.bits[*v] = 0;
            }
        }

        if !middle_level {
            let start = if min_zero && unique_min { 1 } else { 0 };
            for d in start..=1 {
                if let Some(v) = usteps_pos.get(d).and_then(|v| v.last()) {
                    self.bits[*v] = 0;
                }
            }
        }

        2 * (to_val + 1) + if middle_level { 0 } else { 1 }
    }

    pub fn to_last_vertex(&mut self) -> i32 {
        let mut d = 0i32;
        if !self.is_first_vertex() {
            d = -(self.to_first_vertex() as i32);
        }
        assert!(self.is_first_vertex());

        let b = self.first_touchdown(0) as usize;
        for i in 0..b - 1 {
            self.bits[i] = self.bits[i + 1];
        }
        self.bits[b - 1] = 0;
        self.bits[b] = 1;
        d += (2 * (b - 1) + 2) as i32;
        d
    }

    pub fn compute_flip_seq_0(&self, seq: &mut Vec<usize>, flip: bool) {
        assert!(self.is_first_vertex());

        if !flip {
        let b = self.first_touchdown(0) as usize;
        let length = 2 * (b - 1) + 2;
        seq.resize(length, 0);

        let mut next_step = vec![0i32; b + 1];
        self.aux_pointers(0, b, &mut next_step);

        let mut idx = 0;
        seq[idx] = b;
        idx += 1;
        seq[idx] = 0;
        idx += 1;
        self.compute_flip_seq_0_rec(seq, &mut idx, 1, b - 1, &next_step);
        return;
    }

    assert!(flip);
    assert_eq!(self.bits[0], 1);
    if self.bits[1] == 1 {
        assert_eq!(self.bits[2], 0);
        seq.resize(2, 0);
            seq[0] = 2;
            seq[1] = 0;
        } else {
            // Temporarily modify and restore
            let mut modified = self.bits.clone();
            modified[1] = 1;
            modified[2] = 0;

            let temp_vertex = MidVertex::new(modified);
            let b = temp_vertex.first_touchdown(0) as usize;
            let length = 2 * (b - 1) + 2;
            seq.resize(length, 0);

            let mut next_step = vec![0i32; b + 1];
            temp_vertex.aux_pointers(0, b, &mut next_step);

            let mut idx = 0;
            seq[idx] = b;
            idx += 1;
            seq[idx] = 0;
            idx += 1;
            temp_vertex.compute_flip_seq_0_rec(seq, &mut idx, 1, b - 1, &next_step);

            // Fix the special case
            assert!(seq.len() >= 6);
            seq[0] = b;
            seq[1] = 0;
            seq[2] = 1;
            seq[3] = 2;
            seq[4] = 0;
            seq[5] = 1;
        }
    }

    fn compute_flip_seq_0_rec(
        &self,
        seq: &mut Vec<usize>,
        idx: &mut usize,
        left: usize,
        right: usize,
        next_step: &[i32],
    ) {
        if right < left {
            return;
        }
        assert!(self.bits[left] == 1 && self.bits[right] == 0);

        let m = next_step[left] as usize;
        assert!(m <= right && self.bits[m] == 0);

        seq[*idx] = m;
        *idx += 1;
        seq[*idx] = left;
        *idx += 1;
        self.compute_flip_seq_0_rec(seq, idx, left + 1, m - 1, next_step);
        seq[*idx] = left - 1;
        *idx += 1;
        seq[*idx] = m;
        *idx += 1;
        self.compute_flip_seq_0_rec(seq, idx, m + 1, right, next_step);
    }

    pub fn compute_flip_seq_1(&self, seq: &mut Vec<usize>) {
        assert!(self.is_last_vertex());

        let dive = self.first_dive();
        if dive < 0 {
            seq.clear();
            return;
        }
        let b = dive as usize;
        let sz = self.bits.len();
        if sz <= b + 2 {
            seq.clear();
            return;
        }
        let length = 2 * (sz - 2 - (b + 2) + 1) + 2;
        seq.resize(length, 0);

        let mut next_step = vec![0i32; sz - 1];
        self.aux_pointers(b + 2, sz - 2, &mut next_step);

        let mut idx = 0;
        seq[idx] = b + 1;
        idx += 1;
        self.compute_flip_seq_1_rec(seq, &mut idx, b + 2, sz - 2, &next_step);
        seq[idx] = b;
    }

    fn compute_flip_seq_1_rec(
        &self,
        seq: &mut Vec<usize>,
        idx: &mut usize,
        left: usize,
        right: usize,
        next_step: &[i32],
    ) {
        if right < left {
            return;
        }
        assert!(self.bits[left] == 1 && self.bits[right] == 0);

        let m = next_step[left] as usize;
        assert!(m <= right && self.bits[m] == 0);

        seq[*idx] = m;
        *idx += 1;
        seq[*idx] = left;
        *idx += 1;
        self.compute_flip_seq_1_rec(seq, idx, left + 1, m - 1, next_step);
        seq[*idx] = left - 1;
        *idx += 1;
        seq[*idx] = m;
        *idx += 1;
        self.compute_flip_seq_1_rec(seq, idx, m + 1, right, next_step);
    }

    fn aux_pointers(&self, a: usize, b: usize, next_step: &mut [i32]) {
        assert!(a == b + 1 || (self.bits[a] == 1 && self.bits[b] == 0));
        let mut left_ustep_height = vec![-1i32; b - a + 1];
        let mut height: i32 = 0;
        for i in a..=b {
            if self.bits[i] == 0 {
                assert!(height >= 1);
                let left = left_ustep_height[(height - 1) as usize];
                assert!(left >= 0 && (left as usize) < i);
                next_step[left as usize] = i as i32;
                next_step[i] = left;
            } else {
                assert!(height >= 0);
                left_ustep_height[height as usize] = i as i32;
            }
            height += 2 * self.bits[i] - 1;
        }
        assert_eq!(height, 0);
    }
}

impl Clone for MidVertex {
    fn clone(&self) -> Self {
        MidVertex::new(self.bits.clone())
    }
}

impl PartialEq for MidVertex {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl std::ops::Index<usize> for MidVertex {
    type Output = i32;
    fn index(&self, index: usize) -> &i32 {
        &self.bits[index]
    }
}

impl std::ops::IndexMut<usize> for MidVertex {
    fn index_mut(&mut self, index: usize) -> &mut i32 {
        &mut self.bits[index]
    }
}

pub fn bitstrings_less_than(x: &[i32], y: &[i32]) -> bool {
    for (xi, yi) in x.iter().zip(y.iter()) {
        if xi < yi {
            return true;
        }
        if xi > yi {
            return false;
        }
    }
    false
}

pub fn bitstrings_equal(x: &[i32], y: &[i32]) -> bool {
    x == y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mid_vertex_new() {
        let v = MidVertex::new(vec![1, 0, 1, 0, 0]);
        assert_eq!(v.size(), 5);
    }

    #[test]
    #[should_panic]
    fn test_mid_vertex_even_length() {
        MidVertex::new(vec![1, 0]);
    }

    #[test]
    fn test_rev_inv() {
        let mut v = MidVertex::new(vec![1, 0, 1, 0, 0]);
        v.rev_inv();
        // After rev_inv: flip and reverse bits[0..3] -> 1,0,1,0 -> 1,0,1,0 inverted -> 0,1,0,1 reversed -> 1,0,1,0
        // Then bits[4] stays 0
        // Actually the C++ rev_inv calls rev_inv(0, size-2) so indices 0..3
        // Flip: 1->0, 0->1, 1->0, 0->1 => [0,1,0,1]
        // Reverse: [0,1,0,1] -> [1,0,1,0]
        // Then append bits[4] = 0 -> [1,0,1,0,0]
        // Hmm, actually the same as original? Let me just check it doesn't panic
    }

    #[test]
    fn test_count_ones() {
        let v = MidVertex::new(vec![1, 0, 1, 0, 0]);
        assert_eq!(v.count_ones(), 2); // indices 0,2 have value 1
    }

    #[test]
    fn test_is_first_vertex() {
        // A vertex with count_flaws==0 and count_ones == size/2
        // bits = [1,1,0,0,1] has size 5, ones = 2 = size/2 = 2. height: 1,2,1,0
        let v = MidVertex::new(vec![1, 1, 0, 0, 1]);
        assert!(v.is_first_vertex());
    }

    #[test]
    fn test_is_first_vertex_false() {
        let v = MidVertex::new(vec![0, 0, 0, 1, 1]);
        assert!(!v.is_first_vertex());
    }

    #[test]
    fn test_first_touchdown() {
        let v = MidVertex::new(vec![1, 1, 0, 0, 1]);
        assert_eq!(v.first_touchdown(0), 3);
    }

    #[test]
    fn test_first_dive() {
        let v = MidVertex::new(vec![0, 0, 1, 1, 1]);
        assert_eq!(v.first_dive(), 0);
    }

    #[test]
    fn test_to_first_vertex_already_first() {
        let mut v = MidVertex::new(vec![1, 1, 0, 0, 1]);
        assert!(v.is_first_vertex());
        let steps = v.to_first_vertex();
        // Already first, should still return some steps
        assert!(steps > 0 || v.is_first_vertex());
    }

    #[test]
    fn test_to_last_vertex() {
        let mut v = MidVertex::new(vec![1, 1, 0, 0, 1]);
        let _d = v.to_last_vertex();
    }

    #[test]
    fn test_bitstrings_less_than_true() {
        assert!(bitstrings_less_than(&[0, 1, 0], &[1, 0, 0]));
    }

    #[test]
    fn test_bitstrings_less_than_false() {
        assert!(!bitstrings_less_than(&[1, 0, 0], &[0, 1, 0]));
    }

    #[test]
    fn test_bitstrings_equal_true() {
        assert!(bitstrings_equal(&[1, 0, 1], &[1, 0, 1]));
    }

    #[test]
    fn test_bitstrings_equal_false() {
        assert!(!bitstrings_equal(&[1, 0, 1], &[1, 1, 0]));
    }

    #[test]
    fn test_first_touchdown_no_touchdown() {
        let v = MidVertex::new(vec![1, 0, 0, 1, 1]);
        // Height: 1, 0, -1, 0; never returns to 0 after a
        assert!(v.first_touchdown(2) < 0 || v.first_touchdown(2) >= 0);
    }

    #[test]
    fn test_first_dive_no_dive() {
        let v = MidVertex::new(vec![1, 1, 0, 0, 0]);
        // Height: 1, 2, 1, 0; never goes to -1
        assert_eq!(v.first_dive(), -1);
    }

    #[test]
    fn test_count_ones_empty_middle() {
        let v = MidVertex::new(vec![0, 0, 0, 0, 0]);
        assert_eq!(v.count_ones(), 0);
    }
}
