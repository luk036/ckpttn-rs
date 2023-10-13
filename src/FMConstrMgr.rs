// forward declare
use crate::moveinfo::{MoveInfo, MoveInfoV};

/**
 * Check if the move of v can satisfied, GetBetter, or NotStatisfied
 *
 */
enum LegalCheck { NotStatisfied, GetBetter, AllStatisfied }

/**
 * FM Partition Constraint Manager
 */
template <Gnl> class FMConstrMgr {
  private:
    hyprgraph: &Gnl
    f64 bal_tol;
    u32 totalweight{0};
    u32 weight{};  // cache value

  protected:
    Vec<u32> diff;
    u32 lowerbound{};
    u8 num_parts;

    using node_t = Gnl::node_t;

    /**
     * Construct a new FMConstrMgr object
     *
     * @param[in] hyprgraph
     * @param[in] bal_tol
     */
    FMConstrMgr(hyprgraph: &Gnl, f64 bal_tol) : FMConstrMgr(hyprgraph, bal_tol, 2) {}

    /**
     * Construct a new FMConstrMgr object
     *
     * @param[in] hyprgraph
     * @param[in] bal_tol
     * @param[in] num_parts
     */
    FMConstrMgr(hyprgraph: &Gnl, f64 bal_tol, u8 num_parts);

  public:
    /**
     * @brief
     *
     * @param[in] part
     */
    pub fn init(&mut self, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @return LegalCheck
     */
    pub fn check_legal(&mut self, move_info_v: &MoveInfoV<node_t>) -> LegalCheck;

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @return true
     * @return false
     */
    pub fn check_constraints(&mut self, move_info_v: &MoveInfoV<node_t>) -> bool;

    /**
     * @brief
     *
     * @param[in] move_info_v
     */
    pub fn update_move(&mut self, move_info_v: &MoveInfoV<node_t>) -> void;
};
