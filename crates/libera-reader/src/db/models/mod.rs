pub mod books;
pub mod settings;

use anyhow::Result;
pub use books::{
  bookmark::BookBookmarks, bookmark::BookMark, mutool_data::MutoolData, user_data::UserData,
};
use native_db::ToInput;
use native_db::ToKey;
pub use settings::AppTheme;
pub use settings::Settings;
pub use settings::lang::*;
pub use settings::{CardDisplayMode, RootRoute, Route};

use crate::db::DB;

pub(crate) trait GetOrCreate: Sized + ToInput + Clone + Default {
  fn get_or_create(key: impl ToKey, db: &DB) -> Result<Self> {
    Ok(match db.get_primary::<Self>(key)? {
      Some(res) => res,
      None => {
        let item: Self = Self::default();
        db.insert(item.clone()).unwrap();
        item
      }
    })
  }
}
