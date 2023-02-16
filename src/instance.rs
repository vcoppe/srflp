//! This module defines an abstract representation of a SRFLP instance.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrflpInstance {
    pub nb_departments: usize,
    pub lengths: Vec<isize>,
    pub flows: Vec<Vec<isize>>,
}
