use gpui::*;

pub struct PageLinksLayer {
  page_number: usize,
}

impl PageLinksLayer {
  pub fn new(page_number: usize) -> Self {
    Self { page_number }
  }
}

impl Render for PageLinksLayer {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().absolute().inset_0()
  }
}
