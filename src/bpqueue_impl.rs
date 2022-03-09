#include <ckpttn/BPQueue.hpp>
#include <cstdint>  // for i32

/**
 * @brief sentinel
 *
 * @tparam T
 */
// template <typename T, typename Int, class Container>
// Dllink<std::pair<T, Int>> BPQueue<T, Int, Container>::sentinel {};

template class BPQueue<i32, i32>;
// template class BPQueue<i32, i32,
//                FMPmr::Vec<Dllink<std::pair<i32, i32>> > >;
