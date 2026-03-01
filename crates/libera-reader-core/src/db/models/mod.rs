pub mod books;
pub mod settings;

use anyhow::Result;
pub use books::{bookmark::BookMark, mutool_data::MutoolData, thumbnail::Thumbnail, user_data::UserData};
use native_db::ToInput;
use native_db::ToKey;
pub use settings::Settings;
pub use settings::lang::*;
pub use settings::theme::AppTheme;
pub use settings::{RootRoute, Route};

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
