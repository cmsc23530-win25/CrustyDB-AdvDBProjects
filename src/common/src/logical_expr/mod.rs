// Reference: https://github.com/rotaki/decorrelator

mod aggregate;
mod flatmap;
mod hoist;
mod join;
mod logical_rel_expr;
mod map;
mod project;
mod rename;
mod scan;
mod select;

pub mod prelude {
    pub use super::logical_rel_expr::LogicalRelExpr;
    pub use crate::expr::Expression;
    pub use crate::ids::ColumnId;
    pub use crate::join_type::JoinType;
    pub use crate::operation::{AggOp, BinaryOp};
    pub use crate::traits::plan::Plan;
}
