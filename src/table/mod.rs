#[allow(clippy::module_inception)]
mod table;

mod ops;

pub use ops::{PyDropTable, PyRenameTable};
pub use table::PyTable;
