mod normal;
mod in_order;
mod out_of_order;

pub use self::normal::execute as normal;
pub use self::in_order::in_order;
pub use self::out_of_order::execute as out_of_order;
