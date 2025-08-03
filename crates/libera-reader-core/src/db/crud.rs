use anyhow::Result;
use native_db::{db_type, Database, ToInput, ToKey};

pub(crate) fn get_primary<T: ToInput>(key: impl ToKey, db: &Database) -> Result<Option<T>> {
  let r_conn = db.r_transaction()?;
  Ok(r_conn.get().primary(key)?)
}
pub(crate) fn insert<T: ToInput>(item: T, db: &Database) -> db_type::Result<()> {
  let rw_conn = db.rw_transaction()?;
  rw_conn.insert(item)?;
  rw_conn.commit()
}
pub(crate) fn update<T: ToInput>(old_data: T, new_data: T, db: &Database) -> db_type::Result<()> {
  let rw_conn = db.rw_transaction()?;
  rw_conn.update(old_data, new_data)?;
  rw_conn.commit()
}
pub(crate) fn remove<T: ToInput>(item: T, db: &Database) -> Result<T, db_type::Error> {
  let rw_conn = db.rw_transaction()?;
  let res = rw_conn.remove(item)?;
  match rw_conn.commit() {
    Ok(_) => {
      Ok(res)
    }
    Err(e) => {
      Err(e)
    }
  }
}
