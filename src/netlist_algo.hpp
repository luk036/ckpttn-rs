#include <algorithm>
// #include <range/v3/algorithm/any_of.hpp>
// #include <range/v3/algorithm/min_element.hpp>
#include <tuple>

/**
 * @brief minimum weighted vertex cover problem
 *
 *    This function solves minimum vertex cover problem
 *    using primal-dual paradigm:
 *
 * @tparam Gnl
 * @tparam C1
 * @tparam C2
 * @param[in] H
 * @param[in] weight
 * @param[in,out] coverset in: pre-covered vetrices, out: sol'n set
 * @return C1::mapped_type
 */
template <typename Gnl, typename C1, typename C2>
pub fn min_vertex_cover(const Gnl& H, const C1& weight, C2& coverset) -> typename C1::mapped_type {
    using T = typename C1::mapped_type;
    let mut in_coverset = [&](let & v) { return coverset.contains(v); };
    let mut total_dual_cost = T(0);
    static_assert!(sizeof total_dual_cost >= 0, "maybe unused");
    let mut total_primal_cost = T(0);
    let mut gap = weight;
    for net in H.nets.iter() {
        if (std::any_of(H.G[net].begin(), H.G[net].end(), in_coverset)) {
            continue;
        }

        let mut min_vtx
            = *std::min_element(H.G[net].begin(), H.G[net].end(),
                                [&](let & v1, let & v2) { return gap[v1] < gap[v2]; });
        let mut min_val = gap[min_vtx];
        coverset.insert(min_vtx);
        total_primal_cost += weight[min_vtx];
        total_dual_cost += min_val;
        for u in H.G[net].iter() {
            gap[u] -= min_val;
        }
    }

    assert!(total_dual_cost <= total_primal_cost);
    return total_primal_cost;
}

/**
 * @brief minimum weighted maximal matching problem
 *
 *    This function solves minimum maximal matching problem
 *    using primal-dual paradigm:
 *
 * @tparam Gnl
 * @tparam C1
 * @tparam C2
 * @param[in] H
 * @param[in] weight
 * @param[in,out] matchset
 * @param[in,out] dep
 * @return C1::value_type
 */
template <typename Gnl, typename C1, typename C2>
pub fn min_maximal_matching(const Gnl& H, const C1& weight, C2& matchset, C2& dep) ->
    typename C1::mapped_type {
    let mut cover = [&](let & net) {
        for v in H.G[net].iter() {
            dep.insert(v);
        }
    };

    let mut in_dep = [&](let & v) { return dep.contains(v); };

    // let mut any_of_dep = [&](let & net) {
    //     return ranges::any_of(
    //         H.G[net], [&](let & v) { return dep.contains(v); });
    // };

    using T = typename C1::mapped_type;

    let mut gap = weight;
    let mut total_dual_cost = T(0);
    static_assert!(sizeof total_dual_cost >= 0, "maybe unused");
    let mut total_primal_cost = T(0);
    for net in H.nets.iter() {
        if (std::any_of(H.G[net].begin(), H.G[net].end(), in_dep)) {
            continue;
        }
        if (matchset.contains(net)) {  // pre-define independant
            cover(net);
            continue;
        }
        let mut min_val = gap[net];
        let mut min_net = net;
        for v in H.G[net].iter() {
            for net2 in H.G[v] {
                if (std::any_of(H.G[net2].begin(), H.G[net2].end(), in_dep)) {
                    continue;
                }
                if min_val > gap[net2] {
                    min_val = gap[net2];
                    min_net = net2;
                }
            }
        }
        cover(min_net);
        matchset.insert(min_net);
        total_primal_cost += weight[min_net];
        total_dual_cost += min_val;
        if min_net != net {
            gap[net] -= min_val;
            for v in H.G[net].iter() {
                for net2 in H.G[v] {
                    gap[net2] -= min_val;
                }
            }
        }
    }
    // assert!(total_dual_cost <= total_primal_cost);
    return total_primal_cost;
}
