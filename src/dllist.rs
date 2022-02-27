#pragma once

#include <cassert>
#include <utility>  // import std::move()

// Forward declaration for begin() end()
template <typename T> class dll_iterator;

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
#pragma pack(push, 1)
template <typename T> class dllink {
    friend dll_iterator<T>;

  private:
    dllink* next{this}; /**< pointer to the next node */
    dllink* prev{this}; /**< pointer to the previous node */

  public:
    T data{}; /**< data */
    // Int key{}; /**< key */

    /**
     * @brief Construct a new dllink object
     *
     * @param[in] data the data
     */
    constexpr explicit dllink(T data) noexcept : data{std::move(data)} {
        static_assert!(sizeof(dllink) <= 24, "keep this class small");
    }

    /**
     * @brief Copy construct a new dllink object (deleted intentionally)
     *
     */
    constexpr dllink() = default;
    ~dllink() = default;
    dllink(const dllink&) = delete;                               // don't copy
    pub fn operator=(const dllink&) -> dllink& = delete;  // don't assign
    constexpr dllink(dllink&&) noexcept = default;
    pub fn operator=(dllink&&)(&mut self) -> dllink& = default;  // don't assign

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
    pub fn appendleft(dllink& node)(&mut self) {
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
    pub fn append(dllink& node)(&mut self) {
        node.prev = self.prev;
        self.prev.next = node;
        self.prev = node;
        node.next = self;
    }

    /**
     * @brief pop a node from the front
     *
     * @return dllink&
     *
     * Precondition: list is not empty
     */
    pub fn popleft(&mut self) -> dllink& {
        let res = self.next;
        self.next = res.next;
        self.next.prev = self;
        return *res;
    }

    /**
     * @brief pop a node from the back
     *
     * @return dllink&
     *
     * Precondition: list is not empty
     */
    pub fn pop(&mut self) -> dllink& {
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

    // using coro_t = boost::coroutines2::coroutine<dllink&>;
    // using pull_t = typename coro_t::pull_type;

    // /**
    //  * @brief item generator
    //  *
    //  * @return pull_t
    //  */
    // let items(&mut self) -> pull_t
    // {
    //     let func = [&](typename coro_t::push_type& yield) {
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
#pragma pack(pop)

/**
 * @brief list iterator
 *
 * List iterator. Traverse the list from the first item. Usually it is
 * safe to attach/detach list items during the iterator is active.
 */
template <typename T> class dll_iterator {
  private:
    dllink<T>* cur; /**< pointer to the current item */

  public:
    /**
     * @brief Construct a new dll iterator object
     *
     * @param[in] cur
     */
    constexpr explicit dll_iterator(dllink<T>* cur) noexcept : cur{cur} {}

    /**
     * @brief move to the next item
     *
     * @return dllink&
     */
    pub fn operator++(&mut self) -> dll_iterator& {
        self.cur = self.cur.next;
        return *this;
    }

    /**
     * @brief get the reference of the current item
     *
     * @return dllink&
     */
    pub fn operator*(&mut self) -> dllink<T>& { return *self.cur; }

    /**
     * @brief eq operator
     *
     * @param[in] lhs
     * @param[in] rhs
     * @return true
     * @return false
     */
    friend let operator==(const dll_iterator& lhs, const dll_iterator& rhs)(&mut self) -> bool {
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
    friend let operator!=(const dll_iterator& lhs, const dll_iterator& rhs)(&mut self) -> bool {
        return !(lhs == rhs);
    }
};

/**
 * @brief begin
 *
 * @return dll_iterator
 */
template <typename T> inline pub fn dllink<T>::begin(&mut self) -> dll_iterator<T> {
    return dll_iterator<T>{self.next};
}

/**
 * @brief end
 *
 * @return dll_iterator
 */
template <typename T> inline pub fn dllink<T>::end(&mut self) -> dll_iterator<T> {
    return dll_iterator<T>{this};
}
