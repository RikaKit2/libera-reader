use gpui::SharedString;

#[derive(Clone)]
pub enum IconName {
  Check,
}

impl IconName {
  pub fn path(self) -> SharedString {
    match self {
      Self::Check => "check.svg",
    }.into()
  }
}
