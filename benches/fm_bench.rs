use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use netlistx_rs::io::read_are;
use netlistx_rs::{read_node_link_json, Netlist};
use petgraph::graph::NodeIndex;

use ckpttn_rs::fm_bi_constr_mgr::FMBiConstrMgr;
use ckpttn_rs::fm_bi_gain_calc::FMBiGainCalc;
use ckpttn_rs::fm_bi_gain_mgr::FMBiGainMgr;
use ckpttn_rs::netlist_adapter::NetlistHypergraph;
use ckpttn_rs::part_mgr_base::PartMgrBase;
use ckpttn_rs::Hypergraph;

fn get_testcases_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("testcases");
    dir
}

/// Read IBM .netD format with per-line header (C++ readNetD compatible).
fn read_ibm_netd(path: &std::path::Path) -> Netlist {
    let file = File::open(path).expect("cannot open netD file");
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // IBM-PLACE 2.0 format: header on lines 2-5 (line 1 is a dummy "0")
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

fn run_fm_bi(hg: &impl Hypergraph<Node = NodeIndex>, option: bool) {
    let gain_calc = FMBiGainCalc::new(hg, 2);
    let mut gain_mgr = FMBiGainMgr::new(hg, gain_calc, 2);
    gain_mgr.gain_calc.special_handle_2pin_nets = option;
    let constr_mgr = FMBiConstrMgr::new(hg, 0.45);
    let mut part_mgr = PartMgrBase::new(hg, gain_mgr, constr_mgr, 2);
    let mut part = vec![0u8; hg.number_of_modules()];
    part_mgr.legalize(&mut part);
    part_mgr.optimize(&mut part);
}

fn bench_fm_bi_ibm03(c: &mut Criterion) {
    let hg = load_ibm03();
    let mut group = c.benchmark_group("fm_bi_ibm03");
    group.sample_size(10);

    group.bench_function("with_2pin_nets", |b| b.iter(|| run_fm_bi(&hg, true)));
    group.bench_function("without_2pin_nets", |b| b.iter(|| run_fm_bi(&hg, false)));
    group.finish();
}

fn bench_fm_bi_p1(c: &mut Criterion) {
    let hg = load_p1();
    let mut group = c.benchmark_group("fm_bi_p1");
    group.sample_size(10);

    group.bench_function("with_2pin_nets", |b| b.iter(|| run_fm_bi(&hg, true)));
    group.bench_function("without_2pin_nets", |b| b.iter(|| run_fm_bi(&hg, false)));
    group.finish();
}

criterion_group!(benches, bench_fm_bi_ibm03, bench_fm_bi_p1);
criterion_main!(benches);
