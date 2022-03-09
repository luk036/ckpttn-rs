/**
 * @brief doubly linked node (that may also be a "head" a list)
 *
 * A Doubly-linked List class. This class simply contains a link of
 * node's. By adding a "head" node (sentinel), deleting a node is
 * extremely fast (see "Introduction to Algorithm"). This class does
 * not keep the length information as it is not necessary for the FM
 * algorithm. This saves memory and run-time to update the length
 * information. Note that this class does not own the list node. They
 * are supplied by the caller in order to better reuse the nodes.
 */
#[derive(Debug, PartialEq, Eq)]
pub struct Dllink<T> {
    next: *mut Dllink<T>, /**< pointer to the next node */
    prev: *mut Dllink<T>, /**< pointer to the previous node */
    data: T
}

impl<T> Dllink<T> {
    /**
     * @brief Construct a new Dllink object
     *
     * @param[in] data the data
     */
     fn new(&mut self, data: T) -> Self {
         let mut res = Self {
             next: std::ptr::null_mut(),
             prev: std::ptr::null_mut(),
             data
         };
         res.clear();
         res
     }

    /**
     * @brief lock the node (and don't append it to any list)
     *
     */
    pub fn lock(&mut self) { self.next = std::ptr::null_mut(); }

    /**
     * @brief whether the node is locked
     *
     * @return true
     * @return false
     */
    pub fn is_locked(&self) -> bool {
        self.next == std::ptr::null_mut()
    }

    /**
     * @brief whether the list is empty
     *
     * @return true
     * @return false
     */
    pub fn is_empty(&self) -> bool {
        self.next as *const Dllink<T> == self as *const Dllink<T> }

    /**
     * @brief reset the list
     *
     */
    pub fn clear(&mut self) {
        self.next = self as *mut Dllink<T>;
        self.prev = self as *mut Dllink<T>;
    }

    /**
     * @brief detach from a list
     *
     */
    pub fn detach(&mut self) {
        assert!(!self.is_locked());
        let n = self.next;
        let p = self.prev;
        unsafe {
            (*p).next = n;
            (*n).prev = p;
        }
    }

    /**
     * @brief append the node to the front
     *
     * @param[in,out] node
     */
    pub fn appendleft(&mut self, node: &mut Dllink<T>) {
        node.next = self.next;
        unsafe { (*self.next).prev = node as *mut Dllink<T>; }
        self.next = node as *mut Dllink<T>;
        node.prev = self as *mut Dllink<T>;
    }

    /**
     * @brief append the node to the back
     *
     * @param[in,out] node
     */
    pub fn append(&mut self, node: &mut Dllink<T>) {
        node.prev = self.prev;
        unsafe { (*self.prev).next = node as *mut Dllink<T>; }
        self.prev = node as *mut Dllink<T>;
        node.next = self as *mut Dllink<T>;
    }

    /**
     * @brief pop a node from the front
     *
     * @return &mut Dllink<T>
     *
     * Precondition: list is not empty
     */
    pub fn popleft(&mut self) -> &mut Dllink<T> {
        let res = self.next;
        unsafe {
            self.next = (*res).next;
            (*self.next).prev = self as *mut Dllink<T>;
            &mut *res
        }
    }

    /**
     * @brief pop a node from the back
     *
     * @return &mut Dllink<T>
     *
     * Precondition: list is not empty
     */
    pub fn pop(&mut self) -> &mut Dllink<T> {
        let res = self.prev;
        unsafe {
            self.prev = (*res).prev;
            (*self.prev).next = self as *mut Dllink<T>;
            &mut *res
        }
    }
}


/**
 * @brief list iterator
 *
 * List iterator. Traverse the list from the first item. Usually it is
 * safe to attach/detach list items during the iterator is active.
 */
template <typename T> class dll_iterator {
  private:
    Dllink<T>* cur; /**< pointer to the current item */

  public:
    /**
     * @brief Construct a new dll iterator object
     *
     * @param[in] cur
     */
    constexpr explicit dll_iterator(Dllink<T>* cur) noexcept : cur{cur} {}

    /**
     * @brief move to the next item
     *
     * @return &mut Dllink<T>
     */
    pub fn operator++(&mut self) -> dll_iterator& {
        self.cur = self.cur.next;
        return *this;
    }

    /**
     * @brief get the reference of the current item
     *
     * @return &mut Dllink<T>
     */
    pub fn operator*(&mut self) -> &mut Dllink<T> { return *self.cur; }

    /**
     * @brief eq operator
     *
     * @param[in] lhs
     * @param[in] rhs
     * @return true
     * @return false
     */
    pub fn operator==(&mut self, lhs: &dll_iterator, rhs: &dll_iterator) -> bool {
        return lhs.cur == rhs.cur;
    }

    /**
     * @brief neq operator
     *
     * @param[in] lhs
     * @param[in] rhs
     * @return true
     * @return false
     */
    pub fn operator!=(&mut self, lhs: &dll_iterator, rhs: &dll_iterator) -> bool {
        return !(lhs == rhs);
    }
};

/**
 * @brief begin
 *
 * @return dll_iterator
 */
template <typename T> inline pub fn Dllink<T>::begin(&mut self) -> dll_iterator<T> {
    return dll_iterator<T>{self.next};
}

/**
 * @brief end
 *
 * @return dll_iterator
 */
template <typename T> inline pub fn Dllink<T>::end(&mut self) -> dll_iterator<T> {
    return dll_iterator<T>{this};
}
