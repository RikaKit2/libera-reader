use crate::db::DB;
use native_db::{db_type, ToInput, ToKey};

pub fn get_primary<T: ToInput>(key: impl ToKey) -> Option<T> {
  let r_conn = DB.r_transaction().unwrap();
  r_conn.get().primary(key).unwrap()
}

//noinspection RsUnwrap
pub fn insert<T: ToInput>(item: T) -> db_type::Result<()> {
  let rw_conn = DB.rw_transaction().unwrap();
  rw_conn.insert(item).unwrap();
  rw_conn.commit()
}

pub fn insert_batch<T: ToInput>(data: Vec<T>) {
  if data.len() > 0 {
    let rw_conn = DB.rw_transaction().unwrap();
    for i in data {
      rw_conn.insert(i).unwrap();
    }
    rw_conn.commit().unwrap();
  }
}
//noinspection RsUnwrap
pub fn update<T: ToInput>(old_item: T, updated_item: T) -> db_type::Result<()> {
  let rw_conn = DB.rw_transaction().unwrap();
  rw_conn.update(old_item, updated_item).unwrap();
  rw_conn.commit()
}

//noinspection RsUnwrap
pub fn remove<T: ToInput>(item: T) -> Result<T, db_type::Error> {
  let rw_conn = DB.rw_transaction().unwrap();
  let res = rw_conn.remove(item).unwrap();
  match rw_conn.commit() {
    Ok(_) => {
      Ok(res)
    }
    Err(e) => {
      Err(e)
    }
  }
}
