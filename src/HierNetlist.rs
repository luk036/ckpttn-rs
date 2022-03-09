#pragma once

#include <stdint.h>  // for u8

#include <gsl/span>                    // for span
#include <py2cpp/dict.hpp>             // for dict
#include <py2cpp/set.hpp>              // for set
#include <type_traits>                 // for move
#include <Vec>                      // for Vec
#include <xnetwork/classes/graph.hpp>  // for SimpleGraph, Graph, Graph<>::n...

#include "array_like.hpp"  // for shift_array
#include "netlist.hpp"     // for Netlist, Netlist<>::nodeview_t

// using node_t = i32;

// struct PartInfo
// {
//     Vec<u8> part;
//     py::set<node_t> extern_nets;
// };

/**
 * @brief HierNetlist
 *
 * HierNetlist is implemented by xnetwork::Graph, which is a networkx-like graph.
 *
 * @tparam graph_t
 */
template <graph_t> class HierNetlist : public Netlist<graph_t> {
  public:
    using nodeview_t = graph_t::nodeview_t;
    using node_t = graph_t::node_t;
    using index_t = nodeview_t::key_type;

    /* For multi-level algorithms */
    const Netlist<graph_t>* parent;
    Vec<node_t> node_up_map;
    Vec<node_t> node_down_map;
    py::dict<index_t, node_t> cluster_down_map;
    shift_array<Vec<i32>> net_weight{};

    /**
     * @brief Construct a new Hier Netlist object
     *
     * @param[in] gr
     * @param[in] modules
     * @param[in] nets
     */
    HierNetlist(graph_t gr, modules: &nodeview_t, nets: &nodeview_t);

    /**
     * @brief projection down
     *
     * @param[in] part
     * @param[out] part_down
     */
    pub fn projection_down(gsl::span<const u8> part,
                         gsl::span<u8> part_down) const;

    /**
     * @brief projection up
     *
     * @param[in] part
     * @param[out] part_up
     */
    pub fn projection_up(gsl::span<const u8> part, gsl::span<u8> part_up) const;

    /**
     * @brief Get the net weight
     *
     * @return i32
     */
    pub fn get_net_weight(&self, net: &node_t) -> i32 {
        return self.net_weight.empty() ? 1 : self.net_weight[net];
    }
};

template <graph_t>
HierNetlist<graph_t>::HierNetlist(graph_t gr, modules: &nodeview_t, nets: &nodeview_t)
    : Netlist<graph_t>{std::move(gr), modules, nets} {}

// template <graph_t>
// HierNetlist<graph_t>::HierNetlist(graph_t gr, u32 numModules, u32 numNets)
//     : Netlist<graph_t> {std::move(gr), py::range<u32>(numModules),
//           py::range<u32>(numModules, numModules + numNets)}
// {
// }

using SimpleHierNetlist = HierNetlist<xnetwork::SimpleGraph>;
