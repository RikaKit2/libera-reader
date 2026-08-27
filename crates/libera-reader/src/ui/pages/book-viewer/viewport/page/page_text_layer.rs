use gpui::*;

pub struct PageTextLayer {
  page_number: usize,
}

impl PageTextLayer {
  pub fn new(page_number: usize) -> Self {
    Self { page_number }
  }
}

impl Render for PageTextLayer {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().absolute().inset_0()
  }
}
