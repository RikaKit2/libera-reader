use crate::router::{MainRoute, Route};
use crate::side_bar::state::BtnStatus;
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
  pub fn side_bar(&mut self, ctx: &Context) {
    let images1: Vec<(ImageSource, MainRoute)> = vec![
      (include_image!("../../assets/sidebar/heroicons--book-open.svg"), MainRoute::Library),
      (include_image!("../../assets/sidebar/heroicons--folder.svg"), MainRoute::FileManager),
      (include_image!("../../assets/sidebar/heroicons--clock.svg"), MainRoute::History),
      (include_image!("../../assets/sidebar/heroicons--star.svg"), MainRoute::Favorite),
      (include_image!("../../assets/sidebar/heroicons--bookmark.svg"), MainRoute::BookMarks),
    ];
    let images2: Vec<(ImageSource, MainRoute)> = vec![
      (include_image!("../../assets/sidebar/heroicons--chart-bar.svg"), MainRoute::Stats),
      (include_image!("../../assets/sidebar/heroicons--cog-8-tooth.svg"), MainRoute::Settings),
    ];
    let side_bar_frame = egui::containers::Frame {
      inner_margin: Default::default(),
      rounding: Default::default(),
      shadow: Default::default(),
      fill: *BORDER_BASE_COLOR,
      stroke: Default::default(),
      outer_margin: Default::default(),
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

  fn new_btn_layer(&mut self, ui: &mut Ui, btns: Vec<(ImageSource, MainRoute)>) -> f32 {
    let mut layer_y_size: f32 = 0.0;
    let img_size = Vec2::new(28.0, 28.0);

    for (img_source, btn_route) in btns {
      ui.vertical_centered_justified(|ui| {
        let img_color = self.side_bar.btn_colors.get_btn_data_by_route(&btn_route).icon;
        let img = Image::new(img_source).fit_to_exact_size(img_size).tint(img_color);
        let img_btn = ImageButton::new(img);
        let img_response = ui.add(img_btn);

        let btn_data = self.side_bar.btn_colors.get_btn_data_by_route(&btn_route);
        let strip_color = btn_data.strip;
        let rect = Rect::from_min_size(img_response.rect.min, Vec2::new(2.0, 48.0));
        ui.painter().rect_filled(rect, 0.0, strip_color);

        layer_y_size += img_response.intrinsic_size.clone().unwrap().y;
        if img_response.hovered() {
          self.side_bar.btn_colors.change_btn_status(BtnStatus::Hovered, &btn_route);
        } else {
          match &self.route {
            Route::Main(route) => {
              if route != &btn_route {
                self.side_bar.btn_colors.change_btn_status(BtnStatus::Base, &btn_route);
              }
            }
            Route::BookViewer => {}
            Route::Setup => {}
          }
        }
        if img_response.clicked() {
          self.side_bar.btn_colors.change_btn_status(BtnStatus::Clicked, &btn_route);
          self.change_route(Route::Main(btn_route));
        }
      });
    }
    layer_y_size
  }
}
