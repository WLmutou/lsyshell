use eframe::egui;
use crate::shell::lsyshell::LsyShell;
use super::lsyshell::{AuthMethod, DeferredAction, SshConnection};

impl eframe::App for LsyShell {
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
                        if ui.button("首选项").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("退出").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(10.0);
                    ui.menu_button("编辑器", |ui| {
                        if ui.button("新建编辑器").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(10.0);

          
                    ui.menu_button("终端", |ui| {
                        if ui.button("新建终端").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(10.0);
                    
                    ui.menu_button("数据库工具", |ui| {
                        if ui.button("mysql").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("sqlite").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("postgresql").clicked() {
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
    
    
        // // 侧边栏显示连接列表
        egui::SidePanel::left("connections_panel")
            .default_width(180.0)
            .width_range(100.0..=2000.0)
            .resizable(true)
            .show(ctx, |ui | {
                egui::TopBottomPanel::bottom("AB bottom").show_inside(ui, |ui| {
                    ui.label("连接管理");
                });

                ui.heading("连接管理");
                // 添加新建连接按钮（修改后）
                if ui.button("新建连接").clicked() {
                    self.show_connection_dialog();
                }
                // 显示连接列表
            
                // 用于收集需要处理的连接操作
                let mut selected_connection = None;
                let mut connect_target = None;

                // 先遍历连接，收集事件
                for (name, conn) in self.connections.iter() {
                    let response = ui.selectable_label(
                        self.selected_connection.as_deref() == Some(name),
                        &conn.name
                    );

                    if response.double_clicked() {
                        connect_target = Some(name.clone());
                    }

                    if response.clicked() {
                        selected_connection = Some(name.clone());
                    }
                
                }

                // 循环结束后处理状态修改
                if let Some(name) = selected_connection {
                    self.selected_connection = Some(name);
                }
                if let Some(name) = connect_target {
                    self.connect(&name);
                }
                
        });
   
        // 主面板显示终端（修改后）
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SSH终端");
            // 先处理暂存的输入（在闭包外安全修改 self）
            if let Some(cmd) = self.pending_input.take() {
                if  !cmd.is_empty() {
                    let new_cmd = cmd.trim().to_owned();
                    if new_cmd.ends_with("$") {
                        self.send_empty_command();
                    } else {
                        let not_empty_cmd =  new_cmd.replace("$", "");
                        self.send_command(&not_empty_cmd);
                        self.pending_input = Some("".to_string());
                    }
                }
                
            }
            // 显示终端输出
            egui::ScrollArea::vertical().show(ui, |ui| {
               
                let response = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut self.terminal_output)
                        .font(egui::TextStyle::Monospace)
                        .lock_focus(true)
                        // .interactive(self.readonly) // 动态禁用/启用
                        // .cursor_at_end(true)
                );
                let text_edit_id = response.id;
                if self.force_cursor_to_end {
                    // 1.滚动条
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                    // 2.设置光标到最后
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                        let ccursor = egui::text::CCursor::new(self.terminal_output.chars().count());
                        state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                        state.store(ui.ctx(), text_edit_id);
                        ui.ctx().memory_mut(|mem| mem.request_focus(text_edit_id)); // give focus back to the [`TextEdit`].
                    }
                }

                // println!("end ...", response.on);
                // 关键修改：使用 ctx.input() 获取全局输入状态
                // 检测条件：用户按下 Enter 键 + 内容以换行符结尾 + TextEdit 处于焦点
                if response.has_focus(){
                    self.force_cursor_to_end = true;
              
                }

                if ui.input(|i| i.key_pressed(egui::Key::Enter))  {
                    let last_line = self.terminal_output
                    .lines()                    // 按换行符分割为行迭代器
                    .last()                      // 取最后一行
                    .unwrap_or("")               // 处理空内容的情况
                    .to_string();                // 转为 String
                    let cur_cmd = last_line.replace("$", "").trim().to_owned();
                    // println!("cur_cmd:{}",cur_cmd);
                    if cur_cmd.is_empty() {       // 忽略空行提交
                       self.pending_input = Some("$".to_string());
                    } else {
                        self.pending_input = Some(cur_cmd);  // 暂存到 pending_input
                    }
                } else {
                    self.force_cursor_to_end = false; 
                 
                }
               
            });

          
        });
        

        if self.new_dialog.show {
            if let Some(ref mut temp_conn) = self.temp_connection {
                let mut should_save = false;
                let mut should_close = false;
        
                egui::Window::new("新建SSH连接")
                    .open(&mut self.new_dialog.show)
                    .show(ctx, |ui| {
                        // 使用临时连接进行编辑
                        ui.label("名称:");
                        ui.text_edit_singleline(&mut temp_conn.name);
        
                        ui.label("主机:");
                        ui.text_edit_singleline(&mut temp_conn.host);
        
                        ui.label("端口:");
                        ui.add(egui::DragValue::new(&mut temp_conn.port));
        
                        ui.label("用户名:");
                        ui.text_edit_singleline(&mut temp_conn.username);
        
                        match &mut temp_conn.auth_method {
                            AuthMethod::Password(pwd) => {
                                ui.label("密码:");
                                ui.text_edit_singleline(pwd);
                            }
                            AuthMethod::KeyFile(path) => {
                                ui.label("密钥文件路径:");
                                ui.text_edit_singleline(path);
                            }
                        }
        
                        // 按钮处理
                        ui.horizontal(|ui| {
                            if ui.button("保存").clicked() {
                                should_save = true;
                                should_close = true;
                            }
                            if ui.button("取消").clicked() {
                                should_close = true;
                            }
                        });
                    });
        
                // 处理关闭和保存逻辑
                if should_close {
                    self.new_dialog.show = false;
                    if should_save {
                        self.deferred_actions.push(DeferredAction::SaveConnection(temp_conn.clone()));
                    }
                    self.temp_connection = Some(SshConnection::default()); // 重置
                }
            }
             // ！！！在这里处理延迟操作（所有UI渲染完成后）！！！
            while let Some(action) = self.deferred_actions.pop() {
                // ...处理操作...
                match action {
                    DeferredAction::SaveConnection(conn) => {
                                        self.connections.insert(conn.name.clone(), conn);
                                        self.save_connections();
                                    },
                    DeferredAction::DeleteConnection(conn) => {
                        self.connections.remove(conn.name.as_str());
                        self.connection_states.remove(conn.name.as_str()); // 清理对应UI状态
                        self.save_connections();
                    }
                }
            }

        }
    }
   
}

