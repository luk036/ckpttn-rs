use crate::dllist::{Dllink, Dllist};

/// Bounded priority queue
///
/// Bounded Priority Queue with integer keys in [a..b].
/// Implemented by an array (bucket) of doubly-linked lists.
/// Efficient if the keys are bounded by a small integer value.
///
/// Note that this class does not own PQ nodes. This feature
/// allows these nodes sharable in both doubly linked list class and
/// this class. In the FM algorithm, nodes are either attached to
/// the gain buckets (PQ) or to the waitinglist (doubly-linked list),
/// but cannot be in both at the same time.
///
/// Another improvement is to increase the size of the array by one
/// element, i.e. (b - a + 2). The extra dummy array element (called
/// sentinel) is used to reduce the boundary checking during updates.
///
/// All the member functions assume that the keys are inside the bounds.
#[derive(Debug)]
pub struct BPQueue<T> {
    max: usize,
    offset: i32,
    high: usize,
    sentinel: Dllink<(usize, T)>,
    pub bucket: Vec<Dllist<(usize, T)>>,
}

impl<T: Default + Clone> BPQueue<T> {
    /// Construct a new BPQueue object
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// let bpq = BPQueue::<i32>::new(-3, 3);
    ///
    /// assert!(!bpq.bucket[0].is_empty());
    /// assert!(bpq.bucket[1].is_empty());
    /// ```
    pub fn new(a: i32, b: i32) -> Self {
        assert!(a <= b);
        let mut res = Self {
            max: 0,
            offset: a - 1,
            high: (b - a + 1) as usize,
            sentinel: Dllink::new((1314, T::default())),
            bucket: vec![Dllist::new((5354, T::default())); (b - a + 2) as usize],
        };
        for lst in res.bucket.iter_mut() {
            lst.clear();
        }
        res.sentinel.clear();
        res.bucket[0].append(&mut res.sentinel);
        res
    }

    /// Whether the %BPQueue is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// let bpq = BPQueue::<i32>::new(-3, 3);
    ///
    /// assert!(bpq.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.max == 0
    }

    /// Get the max value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// let bpq = BPQueue::<i32>::new(-3, 3);
    ///
    /// assert_eq!(bpq.get_max(), -4);
    /// ```
    pub fn get_max(&self) -> i32 {
        self.offset + self.max as i32
    }

    /// Clear reset the PQ
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// let mut bpq = BPQueue::<i32>::new(-3, 3);
    /// bpq.clear();
    ///
    /// assert!(bpq.is_empty());
    /// ```
    pub fn clear(&mut self) {
        while self.max > 0 {
            self.bucket[self.max].clear();
            self.max -= 1;
        }
    }

    /// Set the key object
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// let mut bpq = BPQueue::<i32>::new(-3, 3);
    ///
    /// assert!(bpq.is_empty());
    /// ```
    pub fn set_key(&mut self, it: &mut Dllink<(usize, T)>, gain: i32) {
        it.data.0 = (gain - self.offset) as usize;
    }

    // /**
    //  * @brief Append item with internal key
    //  *
    //  * @param[in,out] it the item
    //  */
    // pub fn append_direct(&mut self, it: &mut Item)
    //     assert!(static_cast<Int>(it.data.second) > self.offset);
    //     self.append(it, Int(it.data.second));
    // }

    /// Append item with external key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// use ckpttn_rs::dllist::Dllink;
    ///
    /// let mut bpq = BPQueue::<i32>::new(-3, 3);
    /// let mut a = Dllink::<(usize, i32)>::new((0, 3));
    /// bpq.append(&mut a, 0);
    ///
    /// assert!(!bpq.is_empty());
    /// ```
    pub fn append(&mut self, it: &mut Dllink<(usize, T)>, k: i32) {
        assert!(k > self.offset);
        it.data.0 = (k - self.offset) as usize;
        if self.max < it.data.0 {
            self.max = it.data.0;
        }
        self.bucket[it.data.0].append(it);
    }

    /// Pop node with the highest key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// use ckpttn_rs::dllist::Dllink;
    ///
    /// let mut bpq = BPQueue::<i32>::new(-3, 3);
    /// let mut a = Dllink::<(usize, i32)>::new((0, 3));
    /// bpq.append(&mut a, 0);
    /// let (key, v) = bpq.popleft();
    ///
    /// assert_eq!(key, 4);
    /// assert_eq!(v, 3);
    /// ```
    pub fn popleft(&mut self) -> (usize, T) {
        let res = self.bucket[self.max].popleft().data.clone();
        while self.bucket[self.max].is_empty() {
            self.max -= 1;
        }
        res
    }

    /// Detach the item from BPQueue
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// use ckpttn_rs::dllist::Dllink;
    ///
    /// let mut bpq = BPQueue::<i32>::new(-3, 3);
    /// let mut a = Dllink::<(usize, i32)>::new((0, 3));
    /// bpq.append(&mut a, 0);
    /// bpq.detach(&mut a);
    ///
    /// assert!(bpq.is_empty());
    /// ```
    pub fn detach(&mut self, it: &mut Dllink<(usize, T)>) {
        // self.bucket[it.data.second].detach(it)
        it.detach();
        while self.bucket[self.max].is_empty() {
            self.max -= 1;
        }
    }

    /// Decrease key by delta
    ///
    /// Note that the order of items with same key will not be preserved.
    /// For the FM algorithm, this is a desired behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// use ckpttn_rs::dllist::Dllink;
    ///
    /// let mut bpq = BPQueue::<i32>::new(-3, 3);
    /// let mut a = Dllink::<(usize, i32)>::new((0, 3));
    /// bpq.append(&mut a, 0);
    /// bpq.decrease_key(&mut a, 1);
    ///
    /// assert_eq!(bpq.get_max(), -1);
    /// ```
    pub fn decrease_key(&mut self, it: &mut Dllink<(usize, T)>, delta: usize) {
        // self.bucket[it.data.second].detach(it)
        it.detach();
        it.data.0 -= delta;
        assert!(it.data.0 > 0);
        assert!(it.data.0 <= self.high);
        self.bucket[it.data.0].append(it); // FIFO
        if self.max < it.data.0 {
            self.max = it.data.0;
            return;
        }
        while self.bucket[self.max].is_empty() {
            self.max -= 1;
        }
    }

    /// Increase key by delta
    ///
    /// Note that the order of items with same key will not be preserved.
    /// For the FM algorithm, this is a desired behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// use ckpttn_rs::dllist::Dllink;
    ///
    /// let mut bpq = BPQueue::<i32>::new(-3, 3);
    /// let mut a = Dllink::<(usize, i32)>::new((0, 3));
    /// bpq.append(&mut a, 0);
    /// bpq.increase_key(&mut a, 1);
    ///
    /// assert_eq!(bpq.get_max(), 1);
    /// ```
    pub fn increase_key(&mut self, it: &mut Dllink<(usize, T)>, delta: usize) {
        // self.bucket[it.data.second].detach(it)
        it.detach();
        it.data.0 += delta;
        assert!(it.data.0 > 0);
        assert!(it.data.0 <= self.high);
        self.bucket[it.data.0].append(it); // FIFO
        if self.max < it.data.0 {
            self.max = it.data.0;
        }
    }

    /// Modify key by delta
    ///
    /// Note that the order of items with same key will not be preserved.
    /// For the FM algorithm, this is a desired behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ckpttn_rs::bpqueue::BPQueue;
    /// use ckpttn_rs::dllist::Dllink;
    ///
    /// let mut bpq = BPQueue::<i32>::new(-3, 3);
    /// let mut a = Dllink::<(usize, i32)>::new((0, 3));
    /// bpq.append(&mut a, 0);
    /// bpq.modify_key(&mut a, -1);
    ///
    /// assert_eq!(bpq.get_max(), -1);
    /// ```
    pub fn modify_key(&mut self, it: &mut Dllink<(usize, T)>, delta: i32) {
        use core::cmp::Ordering;

        if it.is_locked() {
            return;
        }
        match delta.cmp(&0) {
            Ordering::Greater => self.increase_key(it, delta as usize),
            Ordering::Less => self.decrease_key(it, -delta as usize),
            Ordering::Equal => (),
        }
        // if delta > 0 {
        //     self.increase_key(it, delta as usize);
        // } else if delta < 0 {
        //     self.decrease_key(it, -delta as usize);
        // }
    }
}
