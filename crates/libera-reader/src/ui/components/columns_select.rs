use gpui::*;
use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{select::*, *};
use libera_reader_core::ctx::GlobalCTX;
use rust_i18n::t;

#[derive(Clone, PartialEq)]
struct ColumnOption(u32);

impl ColumnOption {
  fn all() -> Vec<Self> {
    vec![
      Self(2),
      Self(3),
      Self(2),
      Self(4),
      Self(5),
      Self(6),
      Self(7),
      Self(8),
      Self(9),
      Self(10),
      Self(11),
      Self(12),
    ]
  }
}

impl SelectItem for ColumnOption {
  type Value = u32;

  fn title(&self) -> SharedString {
    self.0.to_string().into()
  }

  fn value(&self) -> &Self::Value {
    &self.0
  }
}

pub struct ColumnsSelect {
  columns_select: Entity<SelectState<SearchableVec<ColumnOption>>>,
}
impl ColumnsSelect {
  pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    let columns = SearchableVec::new(ColumnOption::all());
    let current_columns = cx.ctx().settings.read().number_of_columns;
    let initial_index =
      ColumnOption::all().iter().position(|c| c.0 == current_columns).map(|i| IndexPath::default().row(i));
    let columns_select = cx.new(|cx| SelectState::new(columns, initial_index, window, cx).searchable(false));

    cx.new(|cx| {
      fn fun_name(
        _this: &mut ColumnsSelect, _state: &Entity<SelectState<SearchableVec<ColumnOption>>>,
        event: &SelectEvent<SearchableVec<ColumnOption>>, _window: &mut Window,
        cx: &mut Context<'_, ColumnsSelect>,
      ) {
        let SelectEvent::Confirm(new_columns) = event;
        if let Some(new_columns) = new_columns {
          cx.ctx_mut().settings.set_number_of_columns(*new_columns).unwrap();
        };
      }
      cx.subscribe_in(&columns_select, window, fun_name).detach();

      Self { columns_select }
    })
  }
}

impl Render for ColumnsSelect {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().child(
      Select::new(&self.columns_select).placeholder(t!("components.columns_select.placeholder")).w(px(210.0)),
    )
  }
}
