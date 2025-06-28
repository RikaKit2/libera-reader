use crate::ui::components::icon::IconName;
use crate::ui::components::styled::Size;
use crate::ui::utils::adjust_brightness;
use gpui::{div, prelude::FluentBuilder as _, relative, rems, rgb, svg, AnyElement, App, Div, ElementId, InteractiveElement,
           IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window};
use libera_reader_core::types::THEME;

#[derive(IntoElement)]
pub struct Checkbox {
  id: ElementId,
  base: Div,
  label: Option<SharedString>,
  children: Vec<AnyElement>,
  checked: bool,
  disabled: bool,
  size: Size,
  on_click: Option<Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
  theme: THEME,
}

impl Checkbox {
  pub fn new(id: impl Into<ElementId>, theme: THEME) -> Self {
    Self {
      id: id.into(),
      base: div(),
      label: None,
      children: Vec::new(),
      checked: false,
      disabled: false,
      size: Size::default(),
      on_click: None,
      theme,
    }
  }

  pub fn label(mut self, label: impl Into<SharedString>) -> Self {
    self.label = Some(label.into());
    self
  }

  pub fn checked(mut self, checked: bool) -> Self {
    self.checked = checked;
    self
  }

  pub fn on_click(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
    self.on_click = Some(Box::new(handler));
    self
  }
}

impl InteractiveElement for Checkbox {
  fn interactivity(&mut self) -> &mut gpui::Interactivity {
    self.base.interactivity()
  }
}
impl StatefulInteractiveElement for Checkbox {}

impl Styled for Checkbox {
  fn style(&mut self) -> &mut gpui::StyleRefinement {
    self.base.style()
  }
}

impl ParentElement for Checkbox {
  fn extend(&mut self, elements: impl IntoIterator<Item=AnyElement>) {
    self.children.extend(elements);
  }
}

impl RenderOnce for Checkbox {
  fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
    let theme = self.theme.read().unwrap();
    let (color, icon_color) = match self.disabled {
      true => { (adjust_brightness(theme.base_color_content, 0.5), adjust_brightness(theme.primary_content_color, 0.5)) }
      false => { (theme.base_color_content, theme.primary_content_color) }
    };

    div().child(
      self.base
        .id(self.id)
        .flex().flex_row().items_center()
        .gap_2()
        .items_start()
        .line_height(relative(1.))
        .text_color(rgb(theme.base_color_content))
        .map(|this| match self.size {
          Size::XSmall => this.text_xs(),
          Size::Small => this.text_sm(),
          Size::Medium => this.text_base(),
          Size::Large => this.text_lg(),
          _ => this,
        })
        .child(
          div().flex().flex_col()
            .relative()
            .map(|this| match self.size {
              Size::XSmall => this.size_3(),
              Size::Small => this.size_3p5(),
              Size::Medium => this.size_4(),
              Size::Large => this.size(rems(1.125)),
              _ => this.size_4(),
            })
            .flex_shrink_0()
            .border_1()
            .when(!self.checked, |this| {
              this.border_color(rgb(adjust_brightness(theme.base_color_content, 0.4)))
            })
            .map(|this| match self.checked {
              true => this.bg(rgb(color)),
              false => this.bg(gpui::transparent_black()),
            })
            .child(
              svg()
                .absolute()
                .top_px()
                .left_px()
                .map(|this| match self.size {
                  Size::XSmall => this.size_2(),
                  Size::Small => this.size_2p5(),
                  Size::Medium => this.size_3(),
                  Size::Large => this.size_3p5(),
                  _ => this.size_3(),
                })
                .text_color(rgb(icon_color))
                .map(|this| match self.checked {
                  true => this.path(IconName::Check.path()),
                  _ => this,
                }),
            ),
        )
        .when(self.label.is_some() || !self.children.is_empty(), |this| {
          this.child(
            div().flex().flex_col()
              .w_full()
              .line_height(relative(1.2))
              .gap_1()
              .map(|this| {
                if let Some(label) = self.label {
                  this.child(
                    div()
                      .size_full()
                      .text_color(rgb(theme.base_color_content))
                      .line_height(relative(1.))
                      .child(label),
                  )
                } else {
                  this
                }
              })
              .children(self.children),
          )
        })
        .when(self.disabled, |this| {
          this.text_color(rgb(adjust_brightness(theme.base_color_content, 0.5)))
        })
        .when_some(
          self.on_click.filter(|_| !self.disabled),
          |this, on_click| {
            this.on_click(move |_, window, cx| {
              cx.stop_propagation();
              let checked = !self.checked;
              on_click(&checked, window, cx);
            })
          },
        ),
    )
  }
}
