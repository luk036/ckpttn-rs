// #include <__config>  // for std
#include <ckpttn/HierNetlist.hpp>
#include <py2cpp/range.hpp>  // for _iterator, iterable_wrapper

#include "ckpttn/netlist.hpp"  // for Netlist, Netlist<>::nodeview_t

using namespace std;

template <graph_t>
void HierNetlist<graph_t>::projection_up(part: &[u8],
                                         part_up: &mut [u8]) const {
    let & hgr = *self.parent;
    for v in hgr.iter() {
        part_up[self.node_up_map[v]] = part[v];
    }
}

template <graph_t>
void HierNetlist<graph_t>::projection_down(part: &[u8],
                                           part_down: &mut [u8]) const {
    let & hgr = *self.parent;
    for v in self.modules.iter() {
        if self.cluster_down_map.contains(v) {
            let net = self.cluster_down_map.at(v);
            for v2 in hgr.gr[net] {
                part_down[v2] = part[v];
            }
        } else {
            let v2 = self.node_down_map[v];
            part_down[v2] = part[v];
        }
    }
    // if extern_nets.empty() {
    //     return;
    // }
    // extern_nets_down.clear();
    // extern_nets_down.reserve(extern_nets.len());
    // for net in extern_nets.iter() {
    //     extern_nets_down.insert(self.node_down_map[net]);
    // }
}

template class HierNetlist<xnetwork::SimpleGraph>;
