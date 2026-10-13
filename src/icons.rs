#![allow(dead_code)]
use egui::{Color32, Image, Vec2};
use std::sync::{Arc, LazyLock};

#[derive(Clone, Copy)]
pub enum Icon {
    Settings,
    Close,
    Plus,
    Trash,
    Check,
    Route,
    Loader,
}

const ALL: [Icon; 7] = [
    Icon::Settings,
    Icon::Close,
    Icon::Plus,
    Icon::Trash,
    Icon::Check,
    Icon::Route,
    Icon::Loader,
];

impl Icon {
    fn svg(self) -> &'static str {
        match self {
            Icon::Settings => include_str!("../assets/icons/settings.svg"),
            Icon::Close => include_str!("../assets/icons/close.svg"),
            Icon::Plus => include_str!("../assets/icons/plus.svg"),
            Icon::Trash => include_str!("../assets/icons/trash.svg"),
            Icon::Check => include_str!("../assets/icons/check.svg"),
            Icon::Route => include_str!("../assets/icons/route.svg"),
            Icon::Loader => include_str!("../assets/icons/loader.svg"),
        }
    }
    fn uri(self) -> &'static str {
        match self {
            Icon::Settings => "bytes://settings.svg",
            Icon::Close => "bytes://close.svg",
            Icon::Plus => "bytes://plus.svg",
            Icon::Trash => "bytes://trash.svg",
            Icon::Check => "bytes://check.svg",
            Icon::Route => "bytes://route.svg",
            Icon::Loader => "bytes://loader.svg",
        }
    }
}

// currentColor renders black in resvg, so bake in white and tint at draw time.
static CACHE: LazyLock<Vec<Arc<[u8]>>> = LazyLock::new(|| {
    ALL.iter()
        .map(|i| Arc::<[u8]>::from(i.svg().replace("currentColor", "#ffffff").into_bytes()))
        .collect()
});

pub fn image(icon: Icon, size: f32, tint: Color32) -> Image<'static> {
    Image::from_bytes(icon.uri(), CACHE[icon as usize].clone())
        .fit_to_exact_size(Vec2::splat(size))
        .tint(tint)
}
