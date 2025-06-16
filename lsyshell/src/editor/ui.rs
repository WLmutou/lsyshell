use eframe::egui;
use crate::editor::editor::Editor;

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("Menu Bar").show(ctx, |ui| {
            // ui.vertical_centered(|ui| {
            //     ui.heading("Menu Bar");
            // });
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("文件", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                    ui.menu_button("编辑", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                // egui::widgets::global_theme_preference_buttons(ui);
            });
        });
    
        egui::TopBottomPanel::bottom("Status Bar").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Status Bar");
            });
            
        });
    
        egui::SidePanel::left("Activity Bar")
            .max_width(40.0)
            .resizable(false)
            .show(ctx, |ui| ui.add(egui::Label::new("Activity Bar")));
    
        egui::SidePanel::left("Side Bar")
            .default_width(200.0)
            .width_range(200.0..=2000.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::TopBottomPanel::bottom("AB bottom").show_inside(ui, |ui| {
                    ui.label("bottom");
                });
                ui.text_edit_singleline(&mut "Side Bar");
            });
    
        egui::TopBottomPanel::bottom("Panel")
            .default_height(200.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add(egui::TextEdit::multiline(&mut "Panel").desired_rows(10));
            });
    
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Editor");
        });
    }
   
    
    // fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
    //     // TopBottom、Side、TopBottom、Central
    //     // 头部的
    //     egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
    //         // The top panel is often a good place for a menu bar:
    //         egui::menu::bar(ui, |ui| {
    //             // NOTE: no File->Quit on web pages!
    //             let is_web = cfg!(target_arch = "wasm32");
    //             if !is_web {
    //                 ui.menu_button("文件", |ui| {
    //                     if ui.button("Quit").clicked() {
    //                         ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    //                     }
    //                 });
    //                 ui.add_space(16.0);
    //                 ui.menu_button("编辑", |ui| {
    //                     if ui.button("Quit").clicked() {
    //                         ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    //                     }
    //                 });
    //                 ui.add_space(16.0);
    //             }
    //             // egui::widgets::global_theme_preference_buttons(ui);
    //         });
    //     });


    //     egui::SidePanel::left("leftBar").show(ctx, |ui| {
    //         ui.label(format!("frames: {}", self.frames));
    //         ui.heading("计算器 ");
    //         ui.label(format!("count: {}", self.count));
    //         if ui.button("加1").clicked() {
    //             self.count += 1;
    //         }
    //         if ui.button("减1").clicked() {
    //             if self.count > 0 {
    //                 self.count -= 1;
    //             }
    //         }
    //         ui.horizontal(|ui| {
    //             if ui.button("Drak").clicked() {
    //                 ctx.set_visuals(egui::Visuals::dark());
    //             };
    //             if ui.button("Light").clicked() {
    //                 ctx.set_visuals(egui::Visuals::light());
    //             };
    //         });
    //     });

    //     egui::SidePanel::right("panel_name").default_width(300.0).show(ctx, |ui| {
    //         ui.heading("Side Pannel")
    //     });

    //     egui::CentralPanel::default().show(ctx, |ui| {
            
    //         for str in &mut self.strs {
    //             ui.horizontal(|ui| {
    //                 // ui.text_edit_singleline(str);
    //                 if *str != "".to_string() {
    //                     ui.add(egui::TextEdit::singleline(str).hint_text("Please input your name"));
    //                     ui.label("Hello".to_string() + str);
    //                 } else {
    //                     ui.add(egui::TextEdit::singleline(str).hint_text("Please enter your name."))
    //                     .on_hover_text("Your name?");
    //                     ui.label("Please enter your name.");
    //                 }
    //             });
    //         }
    //         if ui.input_mut(|k| {
    //             k.consume_key(egui::Modifiers::CTRL, egui::Key::D)}) 
    //             {
    //                 self.strs.pop();
    //             }
    //         if ui.input_mut(|k| {
    //             k.consume_key(egui::Modifiers::CTRL, egui::Key::N)}) 
    //             {
    //                 self.strs.push("".to_string());
    //             }
    //     });
    //     self.frames += 1;
    // }
}

