//! Forces dynamic linking of the GPUI Kit engine with MiMalloc global allocator.

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui as _;
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_base as _;
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_component as _;
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_component_assets as _;
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_platform as _;
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_kit as _;
