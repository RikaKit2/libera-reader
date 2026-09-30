//! Forces dynamic linking of the GPUI Kit engine with MiMalloc global allocator.

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(not(feature = "dhat-heap"))]
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
use gpui_kit as _;
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_platform as _;
