use crate::router::Route;
use dioxus::prelude::*;
use manganis::mg;

const ACTIVE_BTN: &str = "border-[#ff865b] text-[#9FB9D0]";
const INACTIVE_BTN: &str =
   "border-transparent brightness-50 hover:brightness-75";
const BASE_BTN: &str = "py-[10px] pr-[10px] pl-[8px] outline-none border-l-2";


#[derive(PartialEq, Clone)]
struct BtnData {
    pub url: Route,
    pub icon: String,
}

/* #[component]
fn Btn(i: BtnData) -> Element {
    let cur_route: Route = use_route();
    let active = use_signal(|| false);
    rsx! {
        Link {
            class: if *active.read() { ACTIVE_BTN } else { INACTIVE_BTN },
            onclick: || {},
            to: i.url,
            img { class: "w-[28px] h-[28px]", src: "{i.icon}" }
        }
    }
} */

#[component]
pub(crate) fn SideBar() -> Element {
    #[rustfmt::skip]
  let top_btns: [BtnData; 4] = [
      BtnData { url: Route::Library {}, icon: mg!(file("./assets/icons/sidebar/heroicons--book-open.svg")).to_string() },
      BtnData { url: Route::History {}, icon: mg!(file("./assets/icons/sidebar/heroicons--clock.svg")).to_string() },
      BtnData { url: Route::Favorite {}, icon: mg!(file("./assets/icons/sidebar/heroicons--star.svg")).to_string() },
      BtnData { url: Route::Bookmarks {}, icon: mg!(file("./assets/icons/sidebar/heroicons--bookmark.svg")).to_string() },
  ];

    #[rustfmt::skip]
  let bottom_btns: [BtnData; 2] = [
      BtnData { url: Route::Stats {}, icon: mg!(file("./assets/icons/sidebar/heroicons--chart-bar.svg")).to_string() },
      BtnData { url: Route::Settings {}, icon: mg!(file("./assets/icons/sidebar/heroicons--cog-8-tooth.svg")).to_string() },
  ];

    rsx! {
        div { class: "flex flex-col w-fit bg-[#091319] h-screen fixed z-20 justify-between",
            div { class: "flex flex-col outline-none",
                for i in top_btns {
                    Link { class: BASE_BTN, to: i.url, active_class: ACTIVE_BTN,
                        img { class: "w-[28px] h-[28px]", style: "color: #9FB9D0", src: "{i.icon}" }
                    }
                }
            }
            div { class: "flex flex-col outline-none",
                for i in bottom_btns {
                    Link { class: BASE_BTN, active_class: ACTIVE_BTN, to: i.url,
                        img { class: "w-[28px] h-[28px]", style: "color: #9FB9D0", src: "{i.icon}" }
                    }
                }
            }
        }
    }
}
