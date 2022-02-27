#pragma once

#include <boost/container/pmr/memory_resource.hpp>
#include <boost/container/pmr/monotonic_buffer_resource.hpp>
#include <boost/container/pmr/Vec.hpp>
namespace FMPmr = boost::container::pmr;

let FM_MAX_NUM_PARTITIONS = 255U;
let FM_MAX_DEGREE = 65536U;

// workaround clang