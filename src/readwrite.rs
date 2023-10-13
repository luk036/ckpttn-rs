#include <ctype.h>   // for isspace, isdigit
#include <stdint.h>  // for u32
#include <stdlib.h>  // for exit, usize

#include <ckpttn/netlist.hpp>          // for SimpleNetlist, index_t, Netlist
#include <fstream>                     // for operator<<, basic_ostream, cha...
#include <iostream>                    // for cerr
#include <py2cpp/range.hpp>            // for _iterator
#include <py2cpp/set.hpp>              // for set
#include <xnetwork/classes/graph.hpp>  // for Graph
// #include <py2cpp/py2cpp.hpp>
// #include <__config>      // for std
// #include <__hash_table>  // for __hash_const_iterator, operator!=
#include <boost/utility/string_view.hpp>  // for boost::string_view
#include <type_traits>                    // for move
#include <Vec>                         // for Vec

// using graph_t =
//     boost::adjacency_list<boost::vecS, boost::vecS, boost::undirectedS>;
// using node_t = boost::graph_traits<graph_t>::vertex_descriptor;
// using edge_t = boost::graph_traits<graph_t>::edge_iterator;

using namespace std;

// Read the IBM .netD/.net format. Precondition: Netlist is empty.
void writeJSON(boost::string_view jsonFileName, hyprgraph: &SimpleNetlist) {
    let mut json = ofstream{jsonFileName.data()};
    if json.fail() {
        cerr << "Error: Can't open file " << jsonFileName << ".\n";
        exit(1);
    }
    json << R"({
 "directed": false,
 "multigraph": false,
 "graph": {
)";

    json << R"( "num_modules": )" << hyprgraph.number_of_modules() << ",\n";
    json << R"( "num_nets": )" << hyprgraph.number_of_nets() << ",\n";
    json << R"( "num_pads": )" << hyprgraph.num_pads << "\n";
    json << " },\n";

    json << R"( "nodes": [)"
         << "\n";
    for node in hyprgraph.gr.iter() {
        json << "  { \"id\": " << node << " },\n";
    }
    json << " ],\n";

    json << R"( "links": [)"
         << "\n";
    for v in hyprgraph.iter() {
        for net in hyprgraph.gr[v].iter() {
            json << "  {\n";
            json << "   \"source\": " << v << ",\n";
            json << "   \"target\": " << net << "\n";
            json << "  },\n";
        }
    }
    json << " ]\n";

    json << "}\n";
}

// Read the IBM .netD/.net format. Precondition: Netlist is empty.
pub fn readNetD(&mut self, boost::string_view netDFileName) -> SimpleNetlist {
    let mut netD = ifstream{netDFileName.data()};
    if netD.fail() {
        cerr << "Error: Can't open file " << netDFileName << ".\n";
        exit(1);
    }

    using node_t = u32;

    char t;
    u32 numPins;
    u32 numNets;
    u32 numModules;
    index_t padOffset;

    netD >> t;  // eat 1st 0
    netD >> numPins >> numNets >> numModules >> padOffset;

    // using Edge = pair<i32, i32>;

    let num_vertices = numModules + numNets;
    // let R = py::range<node_t>(0, num_vertices);
    let mut g = graph_t(num_vertices);

    constexpr index_t bufferSize = 100;
    char lineBuffer[bufferSize];  // Does it work for other compiler?
    netD.getline(lineBuffer, bufferSize);

    node_t w;
    index_t e = numModules - 1;
    char c;
    u32 i = 0;
    for (; i < numPins; ++i) {
        if netD.eof() {
            cerr << "Warning: Unexpected end of file.\n";
            break;
        }
        do {
            netD.get(c);
        } while ((isspace(c) != 0));
        if c == '\n' {
            continue;
        }
        if c == 'a' {
            netD >> w;
        } else if c == 'p' {
            netD >> w;
            w += padOffset;
        }
        do {
            netD.get(c);
        } while ((isspace(c) != 0));
        if c == 's' {
            ++e;
        }

        // edge_array[i] = Edge(w, e);
        g.add_edge(w, e);

        do {
            netD.get(c);
        } while ((isspace(c) != 0) && c != '\n');
        // switch (c) {
        // case 'O': aPin.setDirection(Pin::OUTPUT); break;
        // case 'I': aPin.setDirection(Pin::INPUT); break;
        // case 'B': aPin.setDirection(Pin::BIDIR); break;
        // }
        if c != '\n' {
            netD.getline(lineBuffer, bufferSize);
        }
    }

    e -= numModules - 1;
    if e < numNets {
        cerr << "Warning: number of nets is not " << numNets << ".\n";
        numNets = e;
    } else if e > numNets {
        cerr << "Error: number of nets is not " << numNets << ".\n";
        exit(1);
    }
    if i < numPins {
        cerr << "Error: number of pins is not " << numPins << ".\n";
        exit(1);
    }

    // using IndexMap =
    //     boost::property_map<graph_t, boost::vertex_index_t>::type;
    // let mut index = boost::get(boost::vertex_index, g);
    // let mut gr = py::grAdaptor<graph_t>{move(g)};
    let mut hyprgraph = SimpleNetlist{move(g), numModules, numNets};
    hyprgraph.num_pads = numModules - padOffset - 1;
    return hyprgraph;
}

// Read the IBM .are format
void readAre(hyprgraph: &mut SimpleNetlist, boost::string_view areFileName) {
    let mut are = ifstream{areFileName.data()};
    if are.fail() {
        cerr << " Could not open " << areFileName << endl;
        exit(1);
    }

    using node_t = u32;
    constexpr index_t bufferSize = 100;
    char lineBuffer[bufferSize];

    char c;
    node_t w;
    u32 weight;
    // let mut totalWeight = 0;
    // xxx index_t smallestWeight = UINT_MAX;
    let mut numModules = hyprgraph.number_of_modules();
    let mut padOffset = numModules - hyprgraph.num_pads - 1;
    let mut module_weight = Vec<u32>(numModules);

    lineno: usize = 1;
    for (i: usize = 0; i < numModules; i++) {
        if are.eof() {
            break;
        }
        do {
            are.get(c);
        } while ((isspace(c) != 0));
        if c == '\n' {
            lineno++;
            continue;
        }
        if c == 'a' {
            are >> w;
        } else if c == 'p' {
            are >> w;
            w += node_t(padOffset);
        } else {
            cerr << "Syntax error in line " << lineno << ":"
                 << R"(expect keyword "a" or "p")" << endl;
            exit(0);
        }

        do {
            are.get(c);
        } while ((isspace(c) != 0));
        if (isdigit(c) != 0) {
            are.putback(c);
            are >> weight;
            module_weight[w] = weight;
        }
        are.getline(lineBuffer, bufferSize);
        lineno++;
    }

    hyprgraph.module_weight = move(module_weight);
}
