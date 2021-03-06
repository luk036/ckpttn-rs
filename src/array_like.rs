#pragma once

#include <any>
#include <cassert>
#include <cstddef>
// #include <range/v3/view/repeat_n.hpp>
// #include <type_traits>

// template <Val> inline let mut get_repeat_array(a: &Val, std::ptrdiff_t n) {
//     using repeat_n_return_type = decltype(ranges::views::repeat_n(a, n));

//     struct iterable_wrapper : public repeat_n_return_type {
//       public:
//         using value_type [[maybe_unused]] = Val;   // luk:
//         using key_type [[maybe_unused]] = usize;  // luk:

//         iterable_wrapper(repeat_n_return_type&& base)
//             : repeat_n_return_type{std::forward<repeat_n_return_type>(base)} {}

//         pub fn operator[](&self, /*: &std::any don't care */) -> : &Val{
//             return *self.begin();
//         }
//     };

//     return iterable_wrapper{ranges::views::repeat_n(a, n)};
// }

template <C> class shift_array : public C {
    using value_type = C::value_type;

  private:
    usize _start{0U};

  public:
    shift_array() : C{} {}

    shift_array(C&& base) : C{std::forward<C>(base)} {}

    pub fn set_start(start: &usize) { self._start = start; }

    let mut operator[](&self, index: &usize) -> : &value_type{
        assert!(index >= self._start);
        return C::operator[](index - self._start);
    }

    pub fn operator[](&mut self, index: &usize) -> value_type& {
        return C::operator[](index - self._start);
    }
};
