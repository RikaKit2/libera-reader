use crate::types::DB;
use anyhow::Result;
use native_db::{db_type, Database, ToInput, ToKey};
pub(crate) mod book;


pub(crate) fn get_primary<T: ToInput>(key: impl ToKey, db: &Database) -> Result<Option<T>> {
  let r_conn = db.r_transaction()?;
  Ok(r_conn.get().primary(key)?)
}
pub(crate) fn insert<T: ToInput>(item: T, db: &Database) -> db_type::Result<()> {
  let rw_conn = db.rw_transaction()?;
  rw_conn.insert(item)?;
  rw_conn.commit()
}
pub(crate) fn insert_batch<T: ToInput>(data: Vec<T>, db: &Database) -> Result<()> {
  if data.len() > 0 {
    let rw_conn = db.rw_transaction()?;
    for i in data {
      rw_conn.insert(i)?;
    }
    rw_conn.commit()?;
  }
  Ok(())
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

pub(crate) fn update_table<FN, Table: ToInput + Clone>(db: &DB, table: Option<Table>, changing_fn: FN) -> Result<()>
                                                       where FN: FnOnce(&mut Table) {
  let old_table = match table {
    None => { get_primary::<Table>(1, db)?.unwrap() }
    Some(res) => { res }
  };
  let mut new_table = old_table.clone();
  changing_fn(&mut new_table);
  update::<Table>(old_table, new_table, db)?;
  Ok(())
}
