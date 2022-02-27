#pragma once

#include <stdint.h>  // for u32, u8

/**
 * @brief
 *
 * @tparam Node
 */
template <typename Node> struct MoveInfo {
    Node net;
    Node v;
    u8 fromPart;
    u8 toPart;
};

/**
 * @brief
 *
 * @tparam Node
 */
template <typename Node> struct MoveInfoV {
    Node v;
    u8 fromPart;
    u8 toPart;
    // node_t v;
};
