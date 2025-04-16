use crate::router::Route;
use crate::App;
use eframe::egui::ImageButton;
use eframe::emath::Vec2;
use egui::{include_image, Align, Color32, Context, Image, ImageSource, Layout, Rect, Stroke, Ui};
use once_cell::sync::Lazy;
pub(crate) use state::State;

pub(crate) mod state;


pub const BTN_BASE_COLOR: Lazy<Color32> = Lazy::new(|| Color32::from_hex("#4F5C68").unwrap());
pub const BTN_HOVER_COLOR: Lazy<Color32> = Lazy::new(|| Color32::from_hex("#778B9C").unwrap());
pub const BTN_ACTIVE_COLOR: Lazy<Color32> = Lazy::new(|| Color32::from_hex("#9FB9D0").unwrap());
pub const BORDER_ACTIVE_COLOR: Lazy<Color32> = Lazy::new(|| Color32::from_hex("#FF865B").unwrap());
pub const BORDER_BASE_COLOR: Lazy<Color32> = Lazy::new(|| Color32::from_hex("#091319").unwrap());


impl App {
  pub fn make_side_bar(&mut self, ctx: &Context) {
    let images1: Vec<(ImageSource, Route)> = vec![
      (include_image!("icons/heroicons--book-open.svg"), Route::Library),
      (include_image!("icons/heroicons--folder.svg"), Route::FileManager),
      (include_image!("icons/heroicons--clock.svg"), Route::History),
      (include_image!("icons/heroicons--star.svg"), Route::Favorite),
      (include_image!("icons/heroicons--bookmark.svg"), Route::BookMarks),
    ];
    let images2: Vec<(ImageSource, Route)> = vec![
      (include_image!("icons/heroicons--chart-bar.svg"), Route::Stats),
      (include_image!("icons/heroicons--cog-8-tooth.svg"), Route::Settings),
    ];
    let side_bar_frame = egui::containers::Frame {
      inner_margin: Default::default(),
      shadow: Default::default(),
      fill: *BORDER_BASE_COLOR,
      stroke: Default::default(),
      outer_margin: Default::default(),
      corner_radius: Default::default(),
    };
    egui::SidePanel::left("side_bar")
      .resizable(false).frame(side_bar_frame)
      .max_width(48.0)
      .min_width(48.0)
      .show(ctx, |ui| {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.active.weak_bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.active.fg_stroke = Stroke::NONE;
        style.visuals.widgets.active.bg_stroke = Stroke::NONE;
        style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        style.visuals.widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.hovered.fg_stroke = Stroke::NONE;
        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
          let layer_size1 = self.new_btn_layer(ui, images1);
          let between_space = ui.available_size().y - ((layer_size1 / 5.0) * 2.0) - 5.0;
          ui.add_space(between_space);
          self.new_btn_layer(ui, images2);
          ui.add_space(5.0);
        });
      });
  }

  fn new_btn_layer(&mut self, ui: &mut Ui, btns: Vec<(ImageSource, Route)>) -> f32 {
    let mut layer_y_size: f32 = 0.0;
    let img_size = Vec2::new(28.0, 28.0);

    for (img_source, btn_route) in btns {
      ui.vertical_centered_justified(|ui| {
        let img_color = self.side_bar.get_btn(&btn_route).icon;
        let img = Image::new(img_source).fit_to_exact_size(img_size).tint(img_color);
        let img_btn = ImageButton::new(img);
        let img_response = ui.add(img_btn);

        let strip_color = self.side_bar.get_btn(&btn_route).strip;
        let rect_size: Vec2 = Vec2::new(2.0, 37.0);
        let rect = Rect::from_min_size(img_response.rect.min, rect_size);
        ui.painter().rect_filled(rect, 0.0, strip_color);

        layer_y_size += img_response.intrinsic_size.clone().unwrap().y;

        let target_btn = self.side_bar.get_mut_btn(&btn_route);
        if img_response.clicked() {
          target_btn.mark_as_clicked(&mut self.router)
        } else if img_response.hovered() {
          target_btn.mark_as_hovered()
        } else {
          target_btn.remove_mark(&mut self.router)
        }
      });
    }
    layer_y_size
  }
}
