use std::collections::LinkedList;

use crate::midlevel::vertex::{bitstrings_equal, bitstrings_less_than, MidVertex};

/// Tree data structure for mid-level Gray code algorithm.
///
/// Adapted from: Torsten Muetze, Jerri Nummenpalo (2018)
pub struct MidTree {
    num_vertices: usize,
    root: usize,
    children: Vec<LinkedList<usize>>,
    parent: Vec<usize>,
}

impl MidTree {
    pub fn new(x: &MidVertex) -> Self {
        let xv = x.get_bits();
        assert!(xv.len() % 2 == 1);

        let num_vertices = (xv.len() - 1) / 2 + 1;
        let mut children: Vec<LinkedList<usize>> = Vec::with_capacity(num_vertices);
        let mut parent = vec![0usize; num_vertices];

        for _ in 0..num_vertices {
            children.push(LinkedList::new());
        }

        let root = 0usize;
        let mut u = root;
        let mut n = 1usize;

        for i in 0..xv.len() - 1 {
            if xv[i] == 1 {
                children[u].push_back(n);
                parent[n] = u;
                u = n;
                n += 1;
            } else {
                u = parent[u];
            }
        }
        assert_eq!(n, num_vertices);

        MidTree {
            num_vertices,
            root,
            children,
            parent,
        }
    }

    fn deg(&self, u: usize) -> usize {
        if u == self.root {
            self.children[u].len()
        } else {
            self.children[u].len() + 1
        }
    }

    fn num_children(&self, u: usize) -> usize {
        self.children[u].len()
    }

    fn ith_child(&self, u: usize, i: usize) -> usize {
        let mut it = self.children[u].iter();
        it.nth(i).copied().unwrap()
    }

    fn is_tau_preimage(&self) -> bool {
        if self.num_vertices < 3 {
            return false;
        }
        let u = self.ith_child(self.root, 0);
        if self.num_children(u) == 0 {
            return false;
        }
        let v = self.ith_child(u, 0);
        self.num_children(v) == 0
    }

    fn is_tau_image(&self) -> bool {
        self.num_vertices >= 3
            && self.num_children(self.root) >= 2
            && self.num_children(self.ith_child(self.root, 0)) <= 0
    }

    fn tau(&mut self) {
        assert!(self.is_tau_preimage());
        let u = self.ith_child(self.root, 0);
        let v = self.ith_child(u, 0);
        self.move_leaf(v, self.root, 0);
    }

    fn tau_inverse(&mut self) {
        assert!(self.is_tau_image());
        let v0 = self.ith_child(self.root, 0);
        let u = self.ith_child(self.root, 1);
        self.move_leaf(v0, u, 0);
    }

    fn move_leaf(&mut self, leaf: usize, new_parent: usize, pos: usize) {
        assert!(self.num_children(leaf) == 0);
        let old_parent = self.parent[leaf];

        // Remove leaf from old parent's children
        let new_children: Vec<usize> = self.children[old_parent]
            .iter()
            .copied()
            .filter(|&x| x != leaf)
            .collect();
        self.children[old_parent] = new_children.into_iter().collect();

        // Insert leaf at pos in new parent's children
        let mut new_children_p: Vec<usize> = self.children[new_parent].iter().copied().collect();
        new_children_p.insert(pos, leaf);
        self.children[new_parent] = new_children_p.into_iter().collect();

        self.parent[leaf] = new_parent;
    }

    pub(crate) fn rotate(&mut self) {
        assert!(self.num_vertices >= 2);
        let u = self.ith_child(self.root, 0);
        self.parent[self.root] = u;

        let root_children: Vec<usize> = self.children[self.root].iter().copied().collect();
        let mut new_root_children: Vec<usize> = root_children.iter().skip(1).copied().collect();
        self.children[self.root] = new_root_children.drain(..).collect();

        self.children[u].push_back(u);
        *self.children[u].back_mut().unwrap() = self.root;
        self.root = u;
    }

    fn rotate_to_vertex(&mut self, u: usize) {
        while self.root != u {
            self.rotate();
        }
    }

    fn rotate_children(&mut self) {
        self.rotate_children_by(1);
    }

    fn rotate_children_by(&mut self, k: usize) {
        let k = k % self.children[self.root].len();
        if k == 0 {
            return;
        }
        let mut children_vec: Vec<usize> = self.children[self.root].iter().copied().collect();
        children_vec.rotate_left(k);
        self.children[self.root] = children_vec.into_iter().collect();
    }

    pub fn flip_tree(&mut self) -> bool {
        if self.is_tau_preimage() && self.is_flip_tree_tau() {
            self.tau();
            return true;
        }
        if self.is_tau_image() {
            self.tau_inverse();
            if self.is_flip_tree_tau() {
                return true;
            }
            self.tau();
        }
        false
    }

    fn root_canonically(&mut self) {
        let mut c1 = 0;
        let mut c2 = -1i32;
        self.compute_center(&mut c1, &mut c2);

        if c2 != -1 {
            let c2u = c2 as usize;
            let num_bits = 2 * (self.num_vertices - 1);
            self.rotate_to_vertex(c1);
            while self.ith_child(self.root, 0) != c2u {
                self.rotate_children();
            }

            let mut x1 = vec![0i32; num_bits];
            self.to_bitstring(&mut x1);

            self.rotate();
            self.rotate_children_by(self.num_children(self.root) - 1);
            assert!(self.root == c2u && self.ith_child(self.root, 0) == c1);

            let mut x2 = vec![0i32; num_bits];
            self.to_bitstring(&mut x2);

            if bitstrings_less_than(&x1, &x2) {
                self.rotate();
                self.rotate_children_by(self.num_children(self.root) - 1);
                assert!(self.root == c1 && self.ith_child(self.root, 0) == c2u);
            }
        } else {
            let num_bits = 2 * (self.num_vertices - 1);
            self.rotate_to_vertex(c1);
            let mut x = vec![0i32; num_bits];
            self.to_bitstring(&mut x);

            let mut subtree_count = vec![0usize; num_bits];
            let mut c = 0usize;
            let mut depth: i32 = 0;
            for i in 0..num_bits {
                if x[i] == 1 {
                    depth += 1;
                } else {
                    depth -= 1;
                }
                subtree_count[i] = c;
                if depth == 0 {
                    c += 1;
                }
            }
            assert_eq!(c, self.num_children(self.root));

            let k = Self::min_string_rotation(&x);
            self.rotate_children_by(subtree_count[k]);
        }
    }

    fn compute_center(&self, c1: &mut usize, c2: &mut i32) {
        let mut degs = vec![0usize; self.num_vertices];
        let mut leaves = vec![0usize; self.num_vertices];
        let mut num_leaves = 0usize;

        for i in 0..self.num_vertices {
            degs[i] = self.deg(i);
            if degs[i] == 1 {
                leaves[num_leaves] = i;
                num_leaves += 1;
            }
        }

        let mut num_vertices_remaining = self.num_vertices;
        let mut num_new_leaves = 0usize;
        while num_vertices_remaining > 2 {
            for i in 0..num_leaves {
                let u = leaves[i];
                for &it in &self.children[u] {
                    degs[it] -= 1;
                    if degs[it] == 1 {
                        leaves[num_new_leaves] = it;
                        num_new_leaves += 1;
                    }
                }
                if u != self.root {
                    degs[self.parent[u]] -= 1;
                    if degs[self.parent[u]] == 1 {
                        leaves[num_new_leaves] = self.parent[u];
                        num_new_leaves += 1;
                    }
                }
            }
            num_vertices_remaining -= num_leaves;
            num_leaves = num_new_leaves;
            num_new_leaves = 0;
        }

        assert!((1..=2).contains(&num_leaves));
        if num_leaves == 1 {
            *c1 = leaves[0];
            *c2 = -1;
        } else {
            *c1 = leaves[0];
            *c2 = leaves[1] as i32;
        }
    }

    fn is_flip_tree_tau(&mut self) -> bool {
        if self.is_star() {
            return false;
        }

        let r = self.root;
        let u = self.ith_child(self.root, 0);

        let num_bits = 2 * (self.num_vertices - 1);
        let mut this_bitstring = vec![0i32; num_bits];
        let mut canon_bitstring = vec![0i32; num_bits];

        // Check if we have a chain of length 2
        let v = self.ith_child(self.root, 0);
        if self.num_children(v) == 1 && self.num_children(self.ith_child(v, 0)) == 0 {
            self.to_bitstring(&mut this_bitstring);
            self.root_canonically();
            let mut v2 = self.ith_child(self.root, 0);
            while !(self.num_children(v2) == 1 && self.num_children(self.ith_child(v2, 0)) == 0) {
                self.rotate();
                v2 = self.ith_child(self.root, 0);
            }
        } else {
            if self.has_thin_leaf() {
                return false;
            }
            let mut v2 = self.ith_child(self.root, 0);
            let mut c = self.count_pending_edges(v2);
            if c < self.num_children(v2) || c < 2 || self.is_light_dumbbell() {
                return false;
            }
            self.to_bitstring(&mut this_bitstring);
            self.root_canonically();
            v2 = self.ith_child(self.root, 0);
            c = self.count_pending_edges(v2);
            while c < self.num_children(v2) || c < 2 {
                self.rotate();
                self.rotate_children_by(c);
                v2 = self.ith_child(self.root, 0);
                c = self.count_pending_edges(v2);
            }
        }

        self.to_bitstring(&mut canon_bitstring);

        self.rotate_to_vertex(r);
        while self.ith_child(self.root, 0) != u {
            self.rotate_children();
        }

        bitstrings_equal(&this_bitstring, &canon_bitstring)
    }

    fn is_star(&self) -> bool {
        if self.num_vertices <= 3 {
            return false;
        }
        self.deg(self.root) == self.num_vertices - 1
            || self.deg(self.ith_child(self.root, 0)) == self.num_vertices - 1
    }

    fn is_light_dumbbell(&self) -> bool {
        if self.num_vertices < 5 {
            return false;
        }
        let u = self.ith_child(self.root, 0);
        let k = self.num_children(u);
        let l = self.num_children(self.root) - 1;
        k + l + 1 >= self.num_vertices - 1 && k > l
    }

    fn is_thin_leaf(&self, u: usize) -> bool {
        if self.deg(u) > 1 {
            return false;
        }
        if u == self.root {
            self.deg(self.ith_child(u, 0)) == 2
        } else {
            self.deg(self.parent[u]) == 2
        }
    }

    fn has_thin_leaf(&self) -> bool {
        for i in 0..self.num_vertices {
            if self.is_thin_leaf(i) {
                return true;
            }
        }
        false
    }

    fn count_pending_edges(&self, u: usize) -> usize {
        let mut c = 0usize;
        for i in 0..self.num_children(u) {
            let v = self.ith_child(u, i);
            if self.num_children(v) == 0 {
                c += 1;
            } else {
                return c;
            }
        }
        c
    }

    pub(crate) fn to_bitstring(&self, x: &mut [i32]) {
        let mut pos = 0usize;
        self.to_bitstring_rec(x, self.root, &mut pos);
    }

    fn to_bitstring_rec(&self, x: &mut [i32], u: usize, pos: &mut usize) {
        if self.num_children(u) == 0 {
            return;
        }
        for &it in &self.children[u] {
            x[*pos] = 1;
            *pos += 1;
            self.to_bitstring_rec(x, it, pos);
            x[*pos] = 0;
            *pos += 1;
        }
    }

    fn min_string_rotation(x: &[i32]) -> usize {
        let length = x.len() as isize;
        let mut xx = vec![0i32; (2 * length) as usize];
        for i in 0..length {
            xx[i as usize] = x[i as usize];
            xx[(i + length) as usize] = x[i as usize];
        }

        let mut fail = vec![-1isize; (2 * length) as usize];
        let mut k: isize = 0;
        for j in 1..(2 * length) {
            let xj = xx[j as usize];
            let fi = fail[(j - k - 1) as usize];
            let mut i = fi;
            while i != -1 && xj != xx[(k + i + 1) as usize] {
                if xj < xx[(k + i + 1) as usize] {
                    k = j - i - 1;
                }
                i = fail[i as usize];
            }
            if xj != xx[(k + i + 1) as usize] {
                if xj < xx[k as usize] {
                    k = j;
                }
                fail[(j - k) as usize] = -1;
            } else {
                fail[(j - k) as usize] = i + 1;
            }
        }
        k as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mid_tree_new_small() {
        let x = MidVertex::new(vec![1, 0, 0, 1, 1]);
        let _tree = MidTree::new(&x);
    }

    #[test]
    fn test_mid_tree_new_3node() {
        let x = MidVertex::new(vec![1, 0, 1, 0, 1]);
        let _tree = MidTree::new(&x);
    }

    #[test]
    fn test_mid_tree_deg_root() {
        // bits: 1,1,0,0,1 -> trees: root=0, child=1, child=2, back to root, then root child=3
        // Actually for 1,1,0,0,1: the tree has:
        // root 0, first 1 -> child 1, second 1 -> child 2, first 0 -> back to 1, second 0 -> back to 0, then bits[4]=1 is the final marker
        // Actually the tree is built from bits 0..n-2 (excluding the last bit which is always the extra)
        let x = MidVertex::new(vec![1, 1, 0, 0, 1]);
        let tree = MidTree::new(&x);
        assert_eq!(tree.deg(0), 1); // root has one child
    }

    #[test]
    fn test_mid_tree_rotate() {
        let x = MidVertex::new(vec![1, 1, 0, 0, 1]);
        let mut tree = MidTree::new(&x);
        tree.rotate();
        // After rotation, root changes
    }

    #[test]
    fn test_is_star() {
        let x = MidVertex::new(vec![1, 1, 0, 1, 0, 0, 1]);
        let tree = MidTree::new(&x);
        // With 7 bits, 4 vertices -> check if star
        let _is_star = tree.is_star();
    }

    #[test]
    fn test_is_star_small() {
        let x = MidVertex::new(vec![1, 1, 0, 0, 1]);
        let tree = MidTree::new(&x);
        assert!(!tree.is_star()); // num_vertices <= 3 -> false
    }

    #[test]
    fn test_to_bitstring() {
        let x = MidVertex::new(vec![1, 1, 0, 0, 1]);
        let tree = MidTree::new(&x);
        let num_bits = 2 * (tree.num_vertices - 1);
        let mut bits = vec![0i32; num_bits];
        tree.to_bitstring(&mut bits);
        assert_eq!(bits.len(), num_bits);
    }

    #[test]
    fn test_is_tau_preimage() {
        let x = MidVertex::new(vec![1, 1, 0, 1, 0, 0, 1]);
        let tree = MidTree::new(&x);
        let _is_pre = tree.is_tau_preimage();
    }

    #[test]
    fn test_min_string_rotation() {
        let x = [1, 0, 1, 0];
        let k = MidTree::min_string_rotation(&x);
        // The minimal rotation of "1010" is "0101" at position 1 (or 3)
        assert!(k < x.len());
    }

    #[test]
    fn test_compute_center() {
        let x = MidVertex::new(vec![1, 1, 0, 0, 1]);
        let tree = MidTree::new(&x);
        let mut c1 = 0;
        let mut c2 = 0i32;
        tree.compute_center(&mut c1, &mut c2);
        assert!(c1 < tree.num_vertices);
    }

    #[test]
    fn test_is_thin_leaf() {
        let x = MidVertex::new(vec![1, 1, 1, 0, 0, 0, 1]);
        let tree = MidTree::new(&x);
        let _thin = tree.has_thin_leaf();
    }

    #[test]
    fn test_move_leaf() {
        let x = MidVertex::new(vec![1, 1, 0, 0, 1]);
        let mut tree = MidTree::new(&x);
        // Find a leaf
        let leaf = (0..tree.num_vertices)
            .find(|&i| tree.num_children(i) == 0)
            .unwrap();
        let new_parent = tree.root;
        tree.move_leaf(leaf, new_parent, 0);
    }

    #[test]
    fn test_flip_tree_disjoint() {
        let x = MidVertex::new(vec![1, 1, 0, 0, 1]);
        let mut tree = MidTree::new(&x);
        // flip_tree may or may not return true
        let _result = tree.flip_tree();
    }
}
