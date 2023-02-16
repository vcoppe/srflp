use ddo::{StateRanking, WidthHeuristic, SubProblem};

use super::state::SrflpState;


#[derive(Debug, Copy, Clone)]
pub struct SrflpRanking;

impl StateRanking for SrflpRanking {
    type State = SrflpState;

    fn compare(&self, sa: &Self::State, sb: &Self::State) -> std::cmp::Ordering {
        sa.depth.cmp(&sb.depth)
    }
}

pub struct SrflpWidth {
    nb_vars: usize,
    factor: usize,
}
impl SrflpWidth {
    pub fn new(nb_vars: usize, factor: usize) -> SrflpWidth {
        SrflpWidth { nb_vars, factor }
    }
}
impl WidthHeuristic<SrflpState> for SrflpWidth {
    fn max_width(&self, _state: &SubProblem<SrflpState>) -> usize {
        self.nb_vars * self.factor
    }
}
