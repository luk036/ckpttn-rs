#pragma once

// #include "dllist.hpp" // import Dllink
#include <Vec>

#include "FMPmrConfig.hpp"

template <T> class Robin {
  private:
    struct SlNode {
        SlNode* next;
        T key;
    };

    char stack_buf[FM_MAX_NUM_PARTITIONS * sizeof(SlNode)];
    FMPmr::monotonic_buffer_resource rsrc;
    FMPmr::Vec<SlNode> cycle;

    struct iterator {
        SlNode* cur;
        let mut operator!=(&self, other: &iterator) -> bool { return cur != other.cur; }
        let mut operator==(&self, other: &iterator) -> bool { return cur == other.cur; }
        let mut operator++() -> iterator& {
            cur = cur->next;
            return *this;
        }
        let mut operator*(&self) -> : &T{ return cur->key; }
    };

    struct iterable_wrapper {
        Robin<T>* rr;
        T from_part;
        let mut begin() { return iterator{rr->cycle[from_part].next}; }
        let mut end() { return iterator{&rr->cycle[from_part]}; }
        // let mut size(&self) -> usize { return rr->cycle.len() - 1; }
    };

  public:
    pub fn new(T num_parts) { Robin : cycle(num_parts, &rsrc) {
        // num_parts -= 1;
        // for (let mut k = 0U; k != num_parts; ++k)
        // {
        //     self.cycle[k].next = &self.cycle[k + 1];
        //     self.cycle[k].key = k;
        // }
        // self.cycle[num_parts].next = &self.cycle[0];
        // self.cycle[num_parts].key = num_parts;

        auto* slptr = &self.cycle[num_parts - 1];
        let mut k = T(0);
        for sl in self.cycle.iter_mut() {
            sl.key = k;
            slptr->next = &sl;
            slptr = slptr->next;
            ++k;
        }
    }

    pub fn exclude(T from_part) { return iterable_wrapper{this, from_part}; }
};
