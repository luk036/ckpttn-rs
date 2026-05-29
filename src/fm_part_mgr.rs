use crate::part_mgr_base::PartMgrBase;

/// Fiduccia-Mattheyses Partitioning Manager
///
/// Ported from C++ `FMPartMgr` in `FMPartMgr.hpp`.
pub type FMPartMgr<Gnl, GainMgr, ConstrMgr> = PartMgrBase<Gnl, GainMgr, ConstrMgr>;
