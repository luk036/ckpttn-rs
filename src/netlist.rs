#pragma once

#include <stddef.h>  // for usize
#include <stdint.h>  // for u32, u8

#include <py2cpp/dict.hpp>   // for dict
#include <py2cpp/range.hpp>  // for range, _iterator, iterable_wra...
#include <py2cpp/set.hpp>    // for set
#include <type_traits>       // for move
#include <Vec>            // for Vec

// using node_t = i32;

// struct PartInfo
// {
//     Vec<u8> part;
//     py::set<node_t> extern_nets;
// };

/**
 * @brief Netlist
 *
 * Netlist is implemented by xnetwork::Graph, which is a networkx-like graph.
 *
 */
template <typename graph_t> struct Netlist {
    using nodeview_t = typename graph_t::nodeview_t;
    using node_t = typename graph_t::node_t;
    using index_t = typename nodeview_t::key_type;
    // using graph_t = xnetwork::Graph<graph_t>;

    graph_t gr;
    nodeview_t modules;
    nodeview_t nets;
    num_modules: usize{};
    num_nets: usize{};
    num_pads: usize = 0U;
    max_degree: usize{};
    max_net_degree: usize{};
    // u8 cost_model = 0;
    Vec<u32> module_weight;
    bool has_fixed_modules{};
    py::set<node_t> module_fixed;

  public:
    /**
     * @brief Construct a new Netlist object
     *
     * @param[in] gr
     * @param[in] modules
     * @param[in] nets
     */
    Netlist(graph_t gr, modules: &nodeview_t, nets: &nodeview_t);

    /**
     * @brief Construct a new Netlist object
     *
     * @param[in] gr
     * @param[in] numModules
     * @param[in] numNets
     */
    Netlist(graph_t gr, u32 numModules, u32 numNets);

    pub fn begin() const { return self.modules.begin(); }

    pub fn end() const { return self.modules.end(); }

    /**
     * @brief Get the number of modules
     *
     * @return usize
     */
    pub fn number_of_modules(&self) -> usize { return self.num_modules; }

    /**
     * @brief Get the number of nets
     *
     * @return usize
     */
    pub fn number_of_nets(&self) -> usize { return self.num_nets; }

    /**
     * @brief Get the number of nodes
     *
     * @return usize
     */
    pub fn number_of_nodes(&self) -> usize { return self.gr.number_of_nodes(); }

    // /**
    //  * @brief
    //  *
    //  * @return index_t
    //  */
    // let mut number_of_pins(&self) -> index_t { return
    // self.gr.number_of_edges(); }

    /**
     * @brief Get the max degree
     *
     * @return usize
     */
    pub fn get_max_degree(&self) -> usize { return self.max_degree; }

    /**
     * @brief Get the max net degree
     *
     * @return index_t
     */
    pub fn get_max_net_degree(&self) -> usize { return self.max_net_degree; }

    /**
     * @brief Get the module weight
     *
     * @param[in] v
     * @return i32
     */
    pub fn get_module_weight(&self, v: &node_t) -> u32 {
        return self.module_weight.empty() ? 1U : self.module_weight[v];
    }

    /**
     * @brief Get the net weight
     *
     * @return i32
     */
    pub fn get_net_weight(&self, /*net*/: &node_t) -> i32 {
        // return self.net_weight.is_empty() ? 1
        //                                 :
        //                                 self.net_weight[self.net_map[net]];
        return 1;
    }
};

template <typename graph_t>
Netlist<graph_t>::Netlist(graph_t gr, modules: &nodeview_t, nets: &nodeview_t)
    : gr{std::move(gr)},
      modules{modules},
      nets{nets},
      num_modules(modules.size()),
      num_nets(nets.size()) {
    self.has_fixed_modules = (!self.module_fixed.empty());

    // Some compilers does not accept py::range()->iterator as a forward
    // iterator let mut deg_cmp = [this](v: &node_t, w: &node_t) ->
    // index_t {
    //     return self.gr.degree(v) < self.gr.degree(w);
    // };
    // let result1 =
    //     std::max_element(self.modules.begin(), self.modules.end(),
    //     deg_cmp);
    // self.max_degree = self.gr.degree(*result1);
    // let result2 =
    //     std::max_element(self.nets.begin(), self.nets.end(), deg_cmp);
    // self.max_net_degree = self.gr.degree(*result2);

    self.max_degree = 0U;
    for v in self.modules.iter() {
        if self.max_degree < self.gr.degree(v) {
            self.max_degree = self.gr.degree(v);
        }
    }

    self.max_net_degree = 0U;
    for net in self.nets.iter() {
        if self.max_net_degree < self.gr.degree(net) {
            self.max_net_degree = self.gr.degree(net);
        }
    }
}

template <typename graph_t>
Netlist<graph_t>::Netlist(graph_t gr, u32 numModules, u32 numNets)
    : Netlist{std::move(gr), py::range(numModules), py::range(numModules, numModules + numNets)} {}

#include <xnetwork/classes/graph.hpp>  // for Graph, Graph<>::nodeview_t

// using RngIter = decltype(py::range(1));
using graph_t = xnetwork::SimpleGraph;
using index_t = u32;
using SimpleNetlist = Netlist<graph_t>;

template <typename Node> struct Snapshot {
    py::set<Node> extern_nets;
    py::dict<index_t, u8> extern_modules;
};
