//! Everything a typical user needs: `use m08_modules_crates::prelude::*;`

pub use crate::cart::{Cart, CartError};
pub use crate::catalog::{Book, BookError, Catalog, Isbn, IsbnError};
#[cfg(feature = "discounts")]
pub use crate::pricing::Discount;
pub use m08_money::Money;
