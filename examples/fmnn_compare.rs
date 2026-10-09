use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Instant;

use netlistx_rs::io::read_are;
use netlistx_rs::{read_node_link_json, Netlist};

use ckpttn_rs::fm_bi_constr_mgr::FMBiConstrMgr;
use ckpttn_rs::fm_bi_gain_calc::FMBiGainCalc;
use ckpttn_rs::fm_bi_gain_mgr::FMBiGainMgr;
use ckpttn_rs::fm_kway_constr_mgr::FMKWayConstrMgr;
use ckpttn_rs::fm_kway_gain_calc::FMKWayGainCalc;
use ckpttn_rs::fm_kway_gain_mgr::FMKWayGainMgr;
use ckpttn_rs::ml_part_mgr::{MLBiNNPartMgr, MLBiPartMgr, MLKWayNNPartMgr, MLKWayPartMgr};
use ckpttn_rs::netlist_adapter::NetlistHypergraph;
use ckpttn_rs::nn_part_mgr::NNPartMgr;
use ckpttn_rs::part_mgr_base::PartMgrBase;
use ckpttn_rs::Hypergraph;

const BAL_TOL: f64 = 0.45;
const LIMIT_SIZE: usize = 50;
const SEEDS: [u64; 5] = [0, 1, 2, 3, 4];

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

fn make_init(n: usize, k: u8, seed: u64) -> Vec<u8> {
    let mut rng = SplitMix64::new(seed);
    (0..n).map(|_| rng.below(u64::from(k)) as u8).collect()
}

fn get_testcases_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("testcases");
    dir
}

fn read_ibm_netd(path: &std::path::Path) -> Netlist {
    let file = File::open(path).expect("cannot open netD file");
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    let num_pins: u32 = lines[1].trim().parse().expect("numPins");
    let _num_nets: u32 = lines[2].trim().parse().expect("numNets");
    let num_modules: u32 = lines[3].trim().parse().expect("numModules");
    let pad_offset: u32 = lines[4].trim().parse().expect("padOffset");

    let mut netlist = Netlist::new();
    for i in 0..num_modules {
        netlist.add_module(format!("m{}", i)).expect("add_module");
    }

    let mut edge_idx = num_modules;
    let mut pin_count = 0u32;

    for line in &lines[5..] {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if pin_count >= num_pins {
            break;
        }

        let chars: Vec<char> = line.chars().collect();
        let mut pos = 0;

        let node: u32 = if chars[pos] == 'a' {
            pos += 1;
            let num_str: String = chars[pos..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            pos += num_str.len();
            num_str.parse().unwrap_or(0)
        } else if chars[pos] == 'p' {
            pos += 1;
            let num_str: String = chars[pos..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            pos += num_str.len();
            let n: u32 = num_str.parse().unwrap_or(0);
            n + pad_offset
        } else {
            pin_count += 1;
            continue;
        };

        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        if pos < chars.len() && chars[pos] == 's' {
            edge_idx += 1;
        }

        let net_name = format!("n{}", edge_idx - 1 - num_modules);
        if netlist.get_net_by_name(&net_name).is_none() {
            let _ = netlist.add_net(net_name.clone());
        }

        let mod_name = format!("m{}", node);
        if let (Some(net_idx), Some(mod_idx)) = (
            netlist.get_net_by_name(&net_name),
            netlist.get_module_by_name(&mod_name),
        ) {
            let _ = netlist.add_edge(net_idx, mod_idx);
        }
        pin_count += 1;
    }

    netlist
}

fn load_ibm03() -> NetlistHypergraph {
    let mut path = get_testcases_dir();
    path.push("ibm03.net");
    let mut nl = read_ibm_netd(&path);
    let mut are_path = get_testcases_dir();
    are_path.push("ibm03.are");
    read_are(&mut nl, &are_path).expect("Failed to read ibm03.are");
    NetlistHypergraph::from_netlist(&nl)
}

fn load_p1() -> NetlistHypergraph {
    let mut path = get_testcases_dir();
    path.push("p1.json");
    let nl = read_node_link_json(&path).expect("Failed to read p1.json");
    NetlistHypergraph::from_netlist(&nl)
}

fn run_flat(hg: &NetlistHypergraph, part: &mut [u8], k: u8, nn: bool) -> i32 {
    if k == 2 {
        let gain_calc = FMBiGainCalc::new(hg, 2);
        let gain_mgr = FMBiGainMgr::new(hg, gain_calc, 2);
        let constr_mgr = FMBiConstrMgr::new(hg, BAL_TOL);
        if nn {
            let mut part_mgr = NNPartMgr::new(hg, gain_mgr, constr_mgr, 2);
            part_mgr.legalize(part);
            part_mgr.optimize(part);
            part_mgr.total_cost
        } else {
            let mut part_mgr = PartMgrBase::new(hg, gain_mgr, constr_mgr, 2);
            part_mgr.legalize(part);
            part_mgr.optimize(part);
            part_mgr.total_cost
        }
    } else {
        let gain_calc = FMKWayGainCalc::new(hg, k);
        let gain_mgr = FMKWayGainMgr::new(hg, gain_calc, k);
        let constr_mgr = FMKWayConstrMgr::new(hg, BAL_TOL, k);
        if nn {
            let mut part_mgr = NNPartMgr::new(hg, gain_mgr, constr_mgr, usize::from(k));
            part_mgr.legalize(part);
            part_mgr.optimize(part);
            part_mgr.total_cost
        } else {
            let mut part_mgr = PartMgrBase::new(hg, gain_mgr, constr_mgr, usize::from(k));
            part_mgr.legalize(part);
            part_mgr.optimize(part);
            part_mgr.total_cost
        }
    }
}

fn run_ml(hg: &NetlistHypergraph, weights: &[u32], part: &mut [u8], k: u8, nn: bool) -> i32 {
    if k == 2 {
        if nn {
            let mut mgr = MLBiNNPartMgr::new(BAL_TOL);
            mgr.limitsize = LIMIT_SIZE;
            mgr.run_partition(hg, weights, part);
            mgr.total_cost
        } else {
            let mut mgr = MLBiPartMgr::new(BAL_TOL);
            mgr.limitsize = LIMIT_SIZE;
            mgr.run_partition(hg, weights, part);
            mgr.total_cost
        }
    } else if nn {
        let mut mgr = MLKWayNNPartMgr::new(BAL_TOL, k);
        mgr.limitsize = LIMIT_SIZE;
        mgr.run_partition(hg, weights, part);
        mgr.total_cost
    } else {
        let mut mgr = MLKWayPartMgr::new(BAL_TOL, k);
        mgr.limitsize = LIMIT_SIZE;
        mgr.run_partition(hg, weights, part);
        mgr.total_cost
    }
}

fn main() {
    let p1 = load_p1();
    let ibm03 = load_ibm03();
    for (name, hg) in [("p1", &p1), ("ibm03", &ibm03)] {
        let n = hg.number_of_modules();
        let weights: Vec<u32> = hg.modules().map(|v| hg.get_module_weight(v)).collect();
        for k in [2u8, 3u8] {
            for algo in ["FM", "NN"] {
                let nn = algo == "NN";
                for ml in [false, true] {
                    for seed in SEEDS {
                        let mut part = make_init(n, k, seed);
                        let t0 = Instant::now();
                        let cost = if ml {
                            run_ml(hg, &weights, &mut part, k, nn)
                        } else {
                            run_flat(hg, &mut part, k, nn)
                        };
                        let secs = t0.elapsed().as_secs_f64();
                        println!(
                            "{{\"lang\": \"rs\", \"testcase\": \"{name}\", \"k\": {k}, \
                             \"algo\": \"{algo}\", \"ml\": {ml}, \"seed\": {seed}, \
                             \"cost\": {cost}, \"time_s\": {secs:.4}}}"
                        );
                    }
                }
            }
        }
    }
}
