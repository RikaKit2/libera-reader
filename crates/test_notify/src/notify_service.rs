use crossbeam_channel::Receiver;
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Event, EventKind};
use tracing::info;

async fn event_processing(event: Event) {
  match event {
    Event { kind, paths, attrs: _attrs } => {
      match kind {
        EventKind::Create(create_kind) => {
          match create_kind {
            CreateKind::File => {
              let path_to_file = &paths[0].to_str().unwrap();
              info!("new file: {path_to_file}");
            }
            _ => {}
          }
        }
        EventKind::Modify(modify_kind) => {
          match modify_kind {
            ModifyKind::Name(rename_mode) => {
              match rename_mode {
                RenameMode::Both => {
                  let old_path = &paths[0];
                  let new_path = &paths[1];
                  let old_path_str = old_path.to_str().unwrap();
                  let new_path_str = new_path.to_str().unwrap();
                  if new_path.is_dir() {
                    info!("rename dir old_path: {old_path_str}\nnew_path: {new_path_str}");
                  } else {
                    info!("rename file old_path: {old_path_str}\nnew_path: {new_path_str}");
                  }
                }
                RenameMode::From => {}
                RenameMode::To => {}
                _ => {}
              }
            }
            _ => {}
          }
        }
        EventKind::Remove(remove_kind) => {
          match remove_kind {
            RemoveKind::File => {
              let path_to_file = &paths[0].to_str().unwrap();
              info!("this file has been deleted: {path_to_file}");
            }
            RemoveKind::Folder => {
              let path_to_dir = &paths[0].to_str().unwrap();
              info!("this folder has been deleted: {path_to_dir}");
            }
            _ => {}
          }
        }
        _ => {}
      }
    }
  }
}

pub(crate) async fn run(notify_events: Receiver<notify::Result<Event>>) {
  loop {
    match notify_events.try_recv() {
      Ok(res) => {
        match res {
          Ok(event) => {
            event_processing(event).await;
          }
          Err(_) => {}
        }
      }
      Err(_) => {}
    }
  }
}
