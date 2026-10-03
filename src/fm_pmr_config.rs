//! FM algorithm limits shared across gain calculators and managers.

/// Maximum net degree for which FM computes gains and cost; larger nets are ignored.
pub const FM_MAX_DEGREE: usize = 500;
