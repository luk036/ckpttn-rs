use super::dllist;

/**
 * @brief Bounded priority queue
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
 * @tparam _Tp
 * @tparam Int
 * @tparam _Sequence
 * @tparam std::make_unsigned_t<Int>>>>
 */
template <typename _Tp, typename Int = i32,
          typename _Sequence = Vec<dllink<std::pair<_Tp, std::make_unsigned_t<Int>>>>>
pub struct BPQueue {
    using UInt = std::make_unsigned_t<Int>;

    friend bpq_iterator<_Tp, Int>;
    using Item = dllink<std::pair<_Tp, UInt>>;

    static_assert!(std::is_same<Item, typename _Sequence::value_type>::value,
                  "value_type must be the same as the underlying container");

  public:
    using value_type = typename _Sequence::value_type;
    using reference = typename _Sequence::reference;
    using const_reference = typename _Sequence::const_reference;
    using size_type = typename _Sequence::size_type;
    using container_type = _Sequence;

  private:
    Item sentinel{};   //!< sentinel */
    _Sequence bucket;  //!< bucket, array of lists
    UInt max{};        //!< max value
    Int offset;        //!< a - 1
    UInt high;         //!< b - a + 1

  public:
    /**
     * @brief Construct a new BPQueue object
     *
     * @param[in] a lower bound
     * @param[in] b upper bound
     */
    constexpr BPQueue(Int a, Int b)
        : bucket(static_cast<UInt>(b - a) + 2U),
          offset(a - 1),
          high(static_cast<UInt>(b - offset)) {
        assert!(a <= b);
        static_assert!(std::is_integral<Int>::value, "bucket's key must be an integer");
        bucket[0].append(self.sentinel);  // sentinel
    }

    BPQueue(const BPQueue&) = delete;  // don't copy
    ~BPQueue() = default;
    pub fn operator=(const BPQueue&) -> BPQueue& = delete;  // don't assign
    constexpr BPQueue(BPQueue&&) noexcept = default;
    pub fn operator=(BPQueue&&)(&mut self) -> BPQueue& = default;  // don't assign

    /**
     * @brief Whether the %BPQueue is empty.
     *
     * @return true
     * @return false
     */
    pub fn is_empty(&self) -> bool { return self.max == 0U; }

    /**
     * @brief Set the key object
     *
     * @param[out] it the item
     * @param[in] gain the key of it
     */
    pub fn set_key(Item& it, Int gain)(&mut self) {
        it.data.second = static_cast<UInt>(gain - self.offset);
    }

    /**
     * @brief Get the max value
     *
     * @return Int maximum value
     */
    pub fn get_max(&self) -> Int {
        return self.offset + Int(self.max);
    }

    /**
     * @brief Clear reset the PQ
     */
    pub fn clear(&mut self) {
        while self.max > 0 {
            self.bucket[self.max].clear();
            self.max -= 1;
        }
    }

    /**
     * @brief Append item with internal key
     *
     * @param[in,out] it the item
     */
    pub fn append_direct(Item& it)(&mut self) {
        assert!(static_cast<Int>(it.data.second) > self.offset);
        self.append(it, Int(it.data.second));
    }

    /**
     * @brief Append item with external key
     *
     * @param[in,out] it the item
     * @param[in] k  the key
     */
    pub fn append(Item& it, Int k)(&mut self) {
        assert!(k > self.offset);
        it.data.second = UInt(k - self.offset);
        if self.max < it.data.second {
            self.max = it.data.second;
        }
        self.bucket[it.data.second].append(it);
    }

    /**
     * @brief Pop node with the highest key
     *
     * @return dllink&
     */
    pub fn popleft(&mut self) -> Item& {
        auto& res = self.bucket[self.max].popleft();
        while (self.bucket[self.max].is_empty()) {
            self.max -= 1;
        }
        return res;
    }

    /**
     * @brief Decrease key by delta
     *
     * @param[in,out] it the item
     * @param[in] delta the change of the key
     *
     * Note that the order of items with same key will not be preserved.
     * For the FM algorithm, this is a prefered behavior.
     */
    pub fn decrease_key(Item& it, UInt delta)(&mut self) {
        // self.bucket[it.data.second].detach(it)
        it.detach();
        it.data.second -= delta;
        assert!(it.data.second > 0);
        assert!(it.data.second <= self.high);
        self.bucket[it.data.second].append(it);  // FIFO
        if self.max < it.data.second {
            self.max = it.data.second;
            return;
        }
        while self.bucket[self.max].is_empty() {
            self.max -= 1;
        }
    }

    /**
     * @brief Increase key by delta
     *
     * @param[in,out] it the item
     * @param[in] delta the change of the key
     *
     * Note that the order of items with same key will not be preserved.
     * For the FM algorithm, this is a prefered behavior.
     */
    pub fn increase_key(Item& it, UInt delta)(&mut self) {
        // self.bucket[it.data.second].detach(it)
        it.detach();
        it.data.second += delta;
        assert!(it.data.second > 0);
        assert!(it.data.second <= self.high);
        self.bucket[it.data.second].appendleft(it);  // LIFO
        if self.max < it.data.second {
            self.max = it.data.second;
        }
    }

    /**
     * @brief Modify key by delta
     *
     * @param[in,out] it the item
     * @param[in] delta the change of the key
     *
     * Note that the order of items with same key will not be preserved.
     * For FM algorithm, this is a prefered behavior.
     */
    pub fn modify_key(Item& it, Int delta)(&mut self) {
        if (it.is_locked()) {
            return;
        }
        if delta > 0 {
            self.increase_key(it, UInt(delta));
        } else if delta < 0 {
            self.decrease_key(it, UInt(-delta));
        }
    }

    /**
     * @brief Detach the item from BPQueue
     *
     * @param[in,out] it the item
     */
    pub fn detach(Item& it)(&mut self) {
        // self.bucket[it.data.second].detach(it)
        it.detach();
        while (self.bucket[self.max].is_empty()) {
            self.max -= 1;
        }
    }

    /**
     * @brief Iterator point to the begin
     *
     * @return bpq_iterator
     */
    pub fn begin() -> bpq_iterator<_Tp, Int>;

    /**
     * @brief Iterator point to the end
     *
     * @return bpq_iterator
     */
    pub fn end() -> bpq_iterator<_Tp, Int>;
};

/**
 * @brief Bounded Priority Queue Iterator
 *
 * Traverse the queue in descending order.
 * Detaching a queue items may invalidate the iterator because
 * the iterator makes a copy of the current key.
 */
template <typename _Tp, typename Int = i32> class bpq_iterator {
    using UInt = std::make_unsigned_t<Int>;

    // using value_type = _Tp;
    // using key_type = Int;
    using Item = dllink<std::pair<_Tp, UInt>>;

  private:
    BPQueue<_Tp, Int>& bpq;                      //!< the priority queue
    UInt curkey;                                 //!< the current key value
    dll_iterator<std::pair<_Tp, UInt>> curitem;  //!< list iterator pointed to the current item.

    /**
     * @brief Get the reference of the current list
     *
     * @return Item&
     */
    pub fn curlist() -> Item& { return self.bpq.bucket[self.curkey]; }

  public:
    /**
     * @brief Construct a new bpq iterator object
     *
     * @param[in] bpq
     * @param[in] curkey
     */
    constexpr bpq_iterator(BPQueue<_Tp, Int>& bpq, UInt curkey)
        : bpq{bpq}, curkey{curkey}, curitem{bpq.bucket[curkey].begin()} {}

    /**
     * @brief Move to the next item
     *
     * @return bpq_iterator&
     */
    pub fn operator++() -> bpq_iterator& {
        ++self.curitem;
        while (self.curitem == self.curlist().end()) {
            do {
                self.curkey -= 1;
            } while (self.curlist().is_empty());
            self.curitem = self.curlist().begin();
        }
        return *this;
    }

    /**
     * @brief Get the reference of the current item
     *
     * @return Item&
     */
    pub fn operator*() -> Item& { return *self.curitem; }

    /**
     * @brief eq operator
     *
     * @param[in] lhs
     * @param[in] rhs
     * @return true
     * @return false
     */
    friend pub fn operator==(const bpq_iterator& lhs, const bpq_iterator& rhs) -> bool {
        return lhs.curitem == rhs.curitem;
    }

    /**
     * @brief neq operator
     *
     * @param[in] lhs
     * @param[in] rhs
     * @return true
     * @return false
     */
    friend pub fn operator!=(const bpq_iterator& lhs, const bpq_iterator& rhs) -> bool {
        return !(lhs == rhs);
    }
};

/**
 * @brief
 *
 * @return bpq_iterator
 */
template <typename _Tp, typename Int, class _Sequence>
inline pub fn BPQueue<_Tp, Int, _Sequence>::begin() -> bpq_iterator<_Tp, Int> {
    return {*this, self.max};
}

/**
 * @brief
 *
 * @return bpq_iterator
 */
template <typename _Tp, typename Int, class _Sequence>
inline pub fn BPQueue<_Tp, Int, _Sequence>::end() -> bpq_iterator<_Tp, Int> {
    return {*this, 0};
}
