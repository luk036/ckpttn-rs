#pragma once

#include <stdint.h>  // for u32, u8

/**
 * @brief
 *
 * @tparam Node
 */
template <Node> struct MoveInfo {
    Node net;
    Node v;
    u8 from_part;
    u8 to_part;
};

/**
 * @brief
 *
 * @tparam Node
 */
template <Node> struct MoveInfoV {
    Node v;
    u8 from_part;
    u8 to_part;
    // node_t v;
};
