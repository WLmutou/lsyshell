use crate::utils::load_fonts;


#[derive(Default)]
pub struct Editor {
    pub count: u32,
    pub frames: u32,
    pub strs: Vec<String>,
}

impl Editor {
    pub fn default(cc: &eframe::CreationContext) -> Self {
        load_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self { count: 0, frames: 0, strs: vec![] }
    }   
}


