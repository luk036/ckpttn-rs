use crate::dllist::{Dllink, Dllist};

/**
 * Bounded priority queue
 *
 * Bounded Priority Queue with integer keys in [a..b].
 * Implemented by an array (bucket) of doubly-linked lists.
 * Efficient if the keys are bounded by a small integer value.
 *
 * Note that this class does not own PQ nodes. This feature
 * allows these nodes sharable in both doubly linked list class and
 * this class. In the FM algorithm, nodes are either attached to
 * the gain buckets (PQ) or to the waitinglist (doubly-linked list),
 * but cannot be in both at the same time.
 *
 * Another improvement is to increase the size of the array by one
 * element, i.e. (b - a + 2). The extra dummy array element (called
 * sentinel) is used to reduce the boundary checking during updates.
 *
 * All the member functions assume that the keys are inside the bounds.
 *
 */
#[derive(Debug)]
pub struct BPQueue<T> {
    max: usize,
    offset: i32,
    high: usize,
    sentinel: Dllink<(u32, T)>,
    bucket: Vec<Dllist<(u32, T)>>,
}

impl<T: Default + Clone> BPQueue<T> {
    /**
    Construct a new BPQueue object

    # Examples

    ```rust
    use ckpttn_rs::bpqueue::BPQueue;
    let bpq = BPQueue::<i32>::new(-3, 3);
    ```
    */
    pub fn new(a: i32, b: i32) -> Self {
        assert!(a <= b);
        let mut res = Self {
            max: 0,
            offset: a - 1,
            high: (b - a + 1) as usize,
            sentinel: Dllink::<(u32, T)>::new((1314, T::default())),
            bucket: vec![Dllist::<(u32, T)>::new((5354, T::default())); (b - a + 2) as usize],
        };
        for lst in res.bucket.iter_mut() {
            lst.clear();
        }
        res.sentinel.clear();
        res.bucket[0].append(&mut res.sentinel);
        res
    }

    // /**
    //  * @brief Whether the %BPQueue is empty.
    //  *
    //  * @return true
    //  * @return false
    //  */
    // pub fn is_empty(&self) -> bool { return self.max == 0U; }
    //
    // /**
    //  * @brief Set the key object
    //  *
    //  * @param[out] it the item
    //  * @param[in] gain the key of it
    //  */
    // pub fn set_key(&mut self, it: &mut Item, Int gain)
    //     it.data.second = static_cast<UInt>(gain - self.offset);
    // }
    //
    // /**
    //  * @brief Get the max value
    //  *
    //  * @return Int maximum value
    //  */
    // pub fn get_max(&self) -> Int {
    //     return self.offset + Int(self.max);
    // }
    //
    // /**
    //  * @brief Clear reset the PQ
    //  */
    // pub fn clear(&mut self) {
    //     while self.max > 0 {
    //         self.bucket[self.max].clear();
    //         self.max -= 1;
    //     }
    // }
    //
    // /**
    //  * @brief Append item with internal key
    //  *
    //  * @param[in,out] it the item
    //  */
    // pub fn append_direct(&mut self, it: &mut Item)
    //     assert!(static_cast<Int>(it.data.second) > self.offset);
    //     self.append(it, Int(it.data.second));
    // }
    //
    // /**
    //  * @brief Append item with external key
    //  *
    //  * @param[in,out] it the item
    //  * @param[in] k  the key
    //  */
    // pub fn append(&mut self, it: &mut Item, Int k)
    //     assert!(k > self.offset);
    //     it.data.second = UInt(k - self.offset);
    //     if self.max < it.data.second {
    //         self.max = it.data.second;
    //     }
    //     self.bucket[it.data.second].append(it);
    // }
    //
    // /**
    //  * @brief Pop node with the highest key
    //  *
    //  * @return Dllink&
    //  */
    // pub fn popleft(&mut self) -> Item& {
    //     res: &mut auto = self.bucket[self.max].popleft();
    //     while (self.bucket[self.max].is_empty()) {
    //         self.max -= 1;
    //     }
    //     return res;
    // }
    //
    // /**
    //  * @brief Decrease key by delta
    //  *
    //  * @param[in,out] it the item
    //  * @param[in] delta the change of the key
    //  *
    //  * Note that the order of items with same key will not be preserved.
    //  * For the FM algorithm, this is a prefered behavior.
    //  */
    // pub fn decrease_key(&mut self, it: &mut Item, UInt delta)
    //     // self.bucket[it.data.second].detach(it)
    //     it.detach();
    //     it.data.second -= delta;
    //     assert!(it.data.second > 0);
    //     assert!(it.data.second <= self.high);
    //     self.bucket[it.data.second].append(it);  // FIFO
    //     if self.max < it.data.second {
    //         self.max = it.data.second;
    //         return;
    //     }
    //     while self.bucket[self.max].is_empty() {
    //         self.max -= 1;
    //     }
    // }
    //
    // /**
    //  * @brief Increase key by delta
    //  *
    //  * @param[in,out] it the item
    //  * @param[in] delta the change of the key
    //  *
    //  * Note that the order of items with same key will not be preserved.
    //  * For the FM algorithm, this is a prefered behavior.
    //  */
    // pub fn increase_key(&mut self, it: &mut Item, UInt delta)
    //     // self.bucket[it.data.second].detach(it)
    //     it.detach();
    //     it.data.second += delta;
    //     assert!(it.data.second > 0);
    //     assert!(it.data.second <= self.high);
    //     self.bucket[it.data.second].appendleft(it);  // LIFO
    //     if self.max < it.data.second {
    //         self.max = it.data.second;
    //     }
    // }
    //
    // /**
    //  * @brief Modify key by delta
    //  *
    //  * @param[in,out] it the item
    //  * @param[in] delta the change of the key
    //  *
    //  * Note that the order of items with same key will not be preserved.
    //  * For FM algorithm, this is a prefered behavior.
    //  */
    // pub fn modify_key(&mut self, it: &mut Item, Int delta)
    //     if it.is_locked() {
    //         return;
    //     }
    //     if delta > 0 {
    //         self.increase_key(it, UInt(delta));
    //     } else if delta < 0 {
    //         self.decrease_key(it, UInt(-delta));
    //     }
    // }
    //
    // /**
    //  * @brief Detach the item from BPQueue
    //  *
    //  * @param[in,out] it the item
    //  */
    // pub fn detach(&mut self, it: &mut Item)
    //     // self.bucket[it.data.second].detach(it)
    //     it.detach();
    //     while (self.bucket[self.max].is_empty()) {
    //         self.max -= 1;
    //     }
    // }
    //
    // /**
    //  * @brief Iterator point to the begin
    //  *
    //  * @return bpq_iterator
    //  */
    // pub fn begin() -> bpq_iterator<_Tp, Int>;
    //
    // /**
    //  * @brief Iterator point to the end
    //  *
    //  * @return bpq_iterator
    //  */
    // pub fn end() -> bpq_iterator<_Tp, Int>;
}
