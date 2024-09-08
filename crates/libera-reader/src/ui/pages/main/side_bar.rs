use crate::router::MainRoute;
use dioxus::prelude::*;
use manganis::mg;

const ACTIVE_BTN: &str = "border-[#ff865b]";
const INACTIVE_BTN: &str = "border-transparent";
const BASE_BTN: &str = "py-[10px] pr-[10px] pl-[8px] outline-none border-l-2 ";


const INACTIVE_ICON: &str = "brightness-50 hover:brightness-75";
const BASE_ICON: &str = "text-[#9FB9D0] w-[28px] h-[28px] ";

#[derive(PartialEq, Clone)]
struct BtnData {
  pub url: MainRoute,
  pub icon: String,
}

#[component]
fn Btn(i: BtnData, route: Signal<MainRoute>) -> Element {
  let i2 = i.clone();
  rsx! {
    button {
        class: if &i.url == &route() {
            BASE_BTN.to_owned() + ACTIVE_BTN
        } else {
            BASE_BTN.to_owned() + INACTIVE_BTN
        },
        onclick: move |_| route.set(i.url.clone()),
        img {
            style: if &i2.url == &route() {
                BASE_ICON.to_owned()
            } else {
                BASE_ICON.to_owned() + INACTIVE_ICON
            },
            src: "{i.icon}"
        }
    }
}
}


#[component]
pub(crate) fn SideBar(route: Signal<MainRoute>) -> Element {
  #[rustfmt::skip]
  let top_btns: [BtnData; 4] = [
      BtnData { url: MainRoute::Library {}, icon: mg!(file("./assets/icons/sidebar/heroicons--book-open.svg")).to_string() },
      BtnData { url: MainRoute::History {}, icon: mg!(file("./assets/icons/sidebar/heroicons--clock.svg")).to_string() },
      BtnData { url: MainRoute::Favorite {}, icon: mg!(file("./assets/icons/sidebar/heroicons--star.svg")).to_string() },
      BtnData { url: MainRoute::Bookmarks {}, icon: mg!(file("./assets/icons/sidebar/heroicons--bookmark.svg")).to_string() },
  ];

  #[rustfmt::skip]
  let bottom_btns: [BtnData; 2] = [
      BtnData { url: MainRoute::Stats {}, icon: mg!(file("./assets/icons/sidebar/heroicons--chart-bar.svg")).to_string() },
      BtnData { url: MainRoute::Settings {}, icon: mg!(file("./assets/icons/sidebar/heroicons--cog-8-tooth.svg")).to_string() },
  ];

  rsx! {
    div { class: "flex flex-col w-fit bg-[#091319] h-screen fixed z-20 justify-between",
        div { class: "flex flex-col outline-none",
            for i in top_btns {
                Btn { i, route }
            }
        }
        div { class: "flex flex-col outline-none",
            for i in bottom_btns {
                Btn { i, route }
            }
        }
    }
}
}
