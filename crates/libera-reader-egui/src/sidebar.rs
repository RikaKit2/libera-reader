use crate::router::MainRoute;
use crate::side_bar_state::BtnType;
use crate::App;
use eframe::egui::ImageButton;
use egui::{include_image, Align, Color32, Context, Image, ImageSource, Layout, Stroke, Ui, Vec2};

const BASE_BTN_COLOR: &str = "#4F5C68";
const BTN_HOVER_COLOR: &str = "#778B9C";
const BTN_ACTIVE_COLOR: &str = "#9FB9D0";
// const BTN_ACTIVE_BORDER_COLOR: &str = "#FF865B";
const SIDEBAR_BG_COLOR: &str = "#091319";


impl App {
  pub fn sidebar(&mut self, ctx: &Context) {
    let images1 = vec![
      (include_image!("../assets/sidebar/heroicons--book-open.svg"), MainRoute::Library),
      (include_image!("../assets/sidebar/heroicons--folder.svg"), MainRoute::FileManager),
      (include_image!("../assets/sidebar/heroicons--clock.svg"), MainRoute::History),
      (include_image!("../assets/sidebar/heroicons--star.svg"), MainRoute::Favorite),
      (include_image!("../assets/sidebar/heroicons--bookmark.svg"), MainRoute::BookMarks),
    ];
    let images2 = vec![
      (include_image!("../assets/sidebar/heroicons--chart-bar.svg"), MainRoute::State),
      (include_image!("../assets/sidebar/heroicons--cog-8-tooth.svg"), MainRoute::Settings)
    ];

    let side_bar_frame = egui::containers::Frame {
      inner_margin: Default::default(),
      rounding: Default::default(),
      shadow: Default::default(),
      fill: Color32::from_hex(SIDEBAR_BG_COLOR).unwrap(),
      stroke: Default::default(),
      outer_margin: Default::default(),
    };
    egui::SidePanel::left("sidebar")
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
          let layer_size1 = self.get_btn_layer(ui, images1);
          let between_space = ui.available_size().y - ((layer_size1 / 5.0) * 2.0) - 5.0;
          ui.add_space(between_space);
          self.get_btn_layer(ui, images2);
          ui.add_space(5.0);
        });
      });
  }
  fn get_btn_layer(&mut self, ui: &mut Ui, images: Vec<(ImageSource, MainRoute)>) -> f32 {
    let mut layer_y_size: f32 = 0.0;
    for (img_source, btn_route) in images {
      let image_btn = match btn_route {
        MainRoute::Library => { self.get_image_btn(&btn_route, img_source) }
        MainRoute::FileManager => { self.get_image_btn(&btn_route, img_source) }
        MainRoute::History => { self.get_image_btn(&btn_route, img_source) }
        MainRoute::Favorite => { self.get_image_btn(&btn_route, img_source) }
        MainRoute::BookMarks => { self.get_image_btn(&btn_route, img_source) }
        MainRoute::State => { self.get_image_btn(&btn_route, img_source) }
        MainRoute::Settings => { self.get_image_btn(&btn_route, img_source) }
      };
      let response = ui.add(image_btn);
      layer_y_size += response.intrinsic_size.clone().unwrap().y;
      if response.clicked() {
        self.side_bar_state.set_active_btn(btn_route)
      } else if response.hovered() {
        self.side_bar_state.set_hovered_btn(btn_route)
      }
    }
    layer_y_size
  }
  fn get_image_btn<'a>(&self, btn_route: &MainRoute, img_source: ImageSource<'a>) -> ImageButton<'a> {
    let img_color = match self.side_bar_state.get_btn_type(btn_route) {
      BtnType::Hovered => { Color32::from_hex(BTN_HOVER_COLOR).unwrap() }
      BtnType::Active => { Color32::from_hex(BTN_ACTIVE_COLOR).unwrap() }
      BtnType::Empty => { Color32::from_hex(BASE_BTN_COLOR).unwrap() }
    };
    let img = Image::new(img_source).fit_to_exact_size(Vec2::new(28.0, 28.0)).tint(img_color);
    ImageButton::new(img)
  }
}
