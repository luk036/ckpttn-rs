
pub struct Dllink<'a, T> {
    next: &mut Dllink<'a, T>, /**< pointer to the next node */
    prev: &mut Dllink<'a, T>, /**< pointer to the previous node */
    data: T /**< data */
}

impl<'a, T> Dllink<'a, T> {
    /// Create a new Vector2
    #[inline]
    pub const fn new(&mut self, data: T) -> Self {
        Dllink {
            data, 
        }
    }

    /**
     * @brief lock the node (and don't append it to any list)
     *
     */
    pub fn lock(&mut self) { self.next = nullptr; }

    /**
     * @brief whether the node is locked
     *
     * @return true
     * @return false
     */
    pub fn is_locked(&self) -> bool {
        return self.next == nullptr;
    }

    /**
     * @brief whether the list is empty
     *
     * @return true
     * @return false
     */
    pub fn is_empty(&self) -> bool { return self.next == self; }

    /**
     * @brief reset the list
     *
     */
    pub fn clear(&mut self) { self.next = self.prev = self; }

    /**
     * @brief detach from a list
     *
     */
    pub fn detach(&mut self) {
        assert!(!self.is_locked());
        let n = self.next;
        let p = self.prev;
        p.next = n;
        n.prev = p;
    }

    /**
     * @brief append the node to the front
     *
     * @param[in,out] node
     */
    pub fn appendleft(&mut self, node: &mut Dllink)
        node.next = self.next;
        self.next.prev = node;
        self.next = node;
        node.prev = self;
    }

    /**
     * @brief append the node to the back
     *
     * @param[in,out] node
     */
    pub fn append(&mut self, node: &mut Dllink)
        node.prev = self.prev;
        self.prev.next = node;
        self.prev = node;
        node.next = self;
    }

    /**
     * @brief pop a node from the front
     *
     * @return Dllink&
     *
     * Precondition: list is not empty
     */
    pub fn popleft(&mut self) -> Dllink& {
        let res = self.next;
        self.next = res.next;
        self.next.prev = self;
        return *res;
    }

    /**
     * @brief pop a node from the back
     *
     * @return Dllink&
     *
     * Precondition: list is not empty
     */
    pub fn pop(&mut self) -> Dllink& {
        let res = self.prev;
        self.prev = res.prev;
        self.prev.next = self;
        return *res;
    }

    // For iterator

    /**
     * @brief
     *
     * @return dll_iterator
     */
    pub fn begin(&mut self) -> dll_iterator<T>;

    /**
     * @brief
     *
     * @return dll_iterator
     */
    pub fn end(&mut self) -> dll_iterator<T>;

    // using coro_t = boost::coroutines2::coroutine<Dllink&>;
    // using pull_t = coro_t::pull_type;

    // /**
    //  * @brief item generator
    //  *
    //  * @return pull_t
    //  */
    // let items(&mut self) -> pull_t
    // {
    //     let func = [&](coro_t::push_yield: &mut type) {
    //         let cur = self.next;
    //         while (cur != self)
    //         {
    //             yield(*cur);
    //             cur = cur.next;
    //         }
    //     };
    //     return pull_t(func);
    // }
};
