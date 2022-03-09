#include <ckpttn/netlist.hpp>       // for SimpleNetlist, index_t, Netlist
#include <ckpttn/netlist_algo.hpp>  // for min_maximal_matching
// #include <range/v3/all.hpp>
// #include <range/v3/core.hpp>
// #include <range/v3/numeric/accumulate.hpp>
// #include <range/v3/view/enumerate.hpp>
// #include <range/v3/view/remove_if.hpp>
// #include <range/v3/view/transform.hpp>
#include <stdint.h>  // for u32
// #include <__config>                    // for std
// #include <__hash_table>                // for operator!=, __hash_const_iterator
#include <ckpttn/HierNetlist.hpp>      // for SimpleHierNetlist, HierNetlist
#include <memory>                      // for unique_ptr, make_unique
#include <py2cpp/dict.hpp>             // for dict, dict<>::Base
#include <py2cpp/range.hpp>            // for _iterator, iterable_wrapper
#include <py2cpp/set.hpp>              // for set
#include <type_traits>                 // for move
#include <unordered_map>               // for __hash_map_iterator, operator!=
#include <utility>                     // for get
#include <Vec>                      // for Vec<>::iterator, Vec
#include <xnetwork/classes/graph.hpp>  // for Graph, Graph<>::nodeview_t

using node_t = typename SimpleNetlist::node_t;
using namespace std;
// using namespace transrangers;

/**
 * @brief Create a contraction subgraph object
 *
 * @param[in] hgr
 * @param[in] DontSelect
 * @return unique_ptr<SimpleHierNetlist>
 * @todo simplify this function
 */
pub fn create_contraction_subgraph(hgr: &SimpleNetlist, DontSelect: &py::set<node_t>)
    -> unique_ptr<SimpleHierNetlist> {
    let mut weight = py::dict<node_t, u32>{};
    for net in hgr.nets.iter() {
        // weight[net] = accumulate(
        //     transform([&](let & v) { return hgr.get_module_weight(v); }, all(hgr.gr[net])), 0U);
        let mut sum = 0U;
        for v in hgr.gr[net].iter() {
            sum += hgr.get_module_weight(v);
        }
        weight[net] = sum;
    }

    let mut S = py::set<node_t>{};
    let mut dep = DontSelect.copy();
    min_maximal_matching(hgr, weight, S, dep);

    let mut module_up_map = py::dict<node_t, node_t>{};
    module_up_map.reserve(hgr.number_of_modules());
    for v in hgr.iter() {
        module_up_map[v] = v;
    }

    // let mut cluster_map = py::dict<node_t, node_t> {};
    // cluster_map.reserve(S.size());
    let mut node_up_dict = py::dict<node_t, index_t>{};
    let mut net_up_map = py::dict<node_t, index_t>{};

    let mut modules = Vec<node_t>{};
    let mut nets = Vec<node_t>{};
    nets.reserve(hgr.nets.size() - S.size());

    {  // localize C and clusters
        let mut C = py::set<node_t>{};
        let mut clusters = Vec<node_t>{};
        C.reserve(3 * S.size());  // ???
        clusters.reserve(S.size());

        for net in hgr.nets.iter() {
            if S.contains(net) {
                // let mut net_cur = hgr.gr[net].begin();
                // let mut master = *net_cur;
                clusters.push(net);
                for v in hgr.gr[net].iter() {
                    module_up_map[v] = net;
                    C.insert(v);
                }
                // cluster_map[master] = net;
            } else {
                nets.push(net);
            }
        }
        modules.reserve(hgr.modules.size() - C.size() + clusters.size());
        for v in hgr.iter() {
            if C.contains(v) {
                continue;
            }
            modules.push(v);
        }
        modules.insert(modules.end(), clusters.begin(), clusters.end());
    }
    // let mut nodes = Vec<node_t>{};
    // nodes.reserve(modules.size() + nets.size());

    // nodes.insert(nodes.end(), modules.begin(), modules.end());
    // nodes.insert(nodes.end(), nets.begin(), nets.end());
    let mut numModules = u32(modules.size());
    let mut numNets = u32(nets.size());

    {  // localize module_map and net_map
        let mut module_map = py::dict<node_t, index_t>{};
        module_map.reserve(numModules);
        let mut i_v = index_t(0);
        for v in modules.iter() {
            module_map[v] = index_t(i_v);
            ++i_v;
        }

        // let mut net_map = py::dict<node_t, index_t> {};
        net_up_map.reserve(numNets);
        let mut i_net = index_t(0);
        for net in nets.iter() {
            net_up_map[net] = index_t(i_net) + numModules;
            ++i_net;
        }

        node_up_dict.reserve(hgr.number_of_modules());

        for v in hgr.iter() {
            node_up_dict[v] = module_map[module_up_map[v]];
        }
        // for (let & net : nets)
        // {
        //     node_up_dict[net] = net_map[net] + numModules;
        // }
    }

    let mut num_vertices = numModules + numNets;
    // let mut R = py::range<node_t>(0, num_vertices);
    let mut g = graph_t(num_vertices);
    // gr.add_nodes_from(nodes);
    for v in hgr.iter() {
        for net in hgr.gr[v].iter() {
            if S.contains(net) {
                continue;
            }
            g.add_edge(node_up_dict[v], net_up_map[net]);
        }
    }
    // let mut gr = py::grAdaptor<graph_t>(move(g));
    let mut gr = move(g);

    let mut H2 = make_unique<SimpleHierNetlist>(move(gr), py::range(numModules),
                                             py::range(numModules, numModules + numNets));

    let mut node_down_map = Vec<node_t>{};
    node_down_map.resize(numModules);
    // for (let & [v1, v2] : node_up_dict.items())
    for (let & keyvalue : node_up_dict.items()) {
        let mut v1 = get<0>(keyvalue);
        let mut v2 = get<1>(keyvalue);
        node_down_map[v2] = v1;
    }
    let mut cluster_down_map = py::dict<index_t, node_t>{};
    // cluster_down_map.reserve(cluster_map.size()); // ???
    // // for (let & [v, net] : cluster_map.items())
    // for (let & keyvalue : cluster_map.items())
    // {
    //     let mut v = get<0>(keyvalue);
    //     let mut net = get<1>(keyvalue);
    //     cluster_down_map[node_up_dict[v]] = net;
    // }
    for net in S.iter() {
        for v in hgr.gr[net].iter() {
            cluster_down_map[node_up_dict[v]] = net;
        }
    }

    let mut module_weight = Vec<u32>{};
    module_weight.reserve(numModules);
    for (let & i_v : py::range(numModules)) {
        if cluster_down_map.contains(i_v) {
            let net = cluster_down_map[i_v];
            module_weight.push(weight[net]);
        } else {
            let v2 = node_down_map[i_v];
            module_weight.push(hgr.get_module_weight(v2));
        }
    }

    // if isinstance(hgr.modules, range):
    //     node_up_map = [0 for _ in hgr.modules]
    // elif isinstance(hgr.modules, list):
    //     node_up_map = {}
    // else:
    //     raise NotImplementedError
    let mut node_up_map = Vec<node_t>(hgr.modules.size());
    for v in hgr.modules.iter() {
        node_up_map[v] = node_up_dict[v];
    }

    H2->node_up_map = move(node_up_map);
    H2->node_down_map = move(node_down_map);
    H2->cluster_down_map = move(cluster_down_map);
    H2->module_weight = move(module_weight);
    H2->parent = &hgr;
    return H2;
}
