use crate::utils::load_fonts;

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use ssh2::{Channel, Session};
use std::net::TcpStream;
use std::path::Path;

// use std::sync::mpsc::{channel, Receiver, Sender};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::thread;
use super::lsyerror::LsyError;
use super::terminal::TerminalEmulator;


// SSH 连接配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
}
impl Default for SshConnection {
    fn default() -> Self {
       Self{
        name: "New Connection".into(),
        host: "localhost".into(),
        port: 22,
        username: "root".into(),
        auth_method: AuthMethod::Password("".into()),

       }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewConnectionDialog {
    pub show: bool,
    pub connection: SshConnection,
}


impl Default for NewConnectionDialog {
    fn default() -> Self {
        Self {
            show: false,
            connection: SshConnection::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Password(String),
    KeyFile(String),
}

pub enum DeferredAction {
    SaveConnection(SshConnection),
    DeleteConnection(SshConnection),
}

#[derive(Default)]
pub struct ConnectionState {
    pub context_menu: Option<egui::Pos2>, // 菜单位置
    pub is_editing: bool,                 // 编辑状态
    pub is_selected: bool,                // 选中状态
}

pub struct LsyShell {
    pub connections: HashMap<String, SshConnection>,
    pub selected_connection: Option<String>,
    pub new_dialog: NewConnectionDialog,  // 替换原来的 new_connection 和 show_new_dialog
    pub session: Option<Session>,
    pub terminal_output: String,
    pub force_cursor_to_end: bool,  // 光标是否要设置到最下面和最后面
    pub pending_input: Option<String>,  // 新增：临时存储待处理的输入
    pub command_history: Vec<String>,
    pub history_index: usize,
    pub pending_commands: Vec<String>, // 新增命令队列 
    pub deferred_actions: Vec<DeferredAction>, // 新增延迟操作队列
    pub temp_connection: Option<SshConnection>, // 新增临时连接存储
    pub connection_states: HashMap<String, ConnectionState>, // 每个连接的UI状态

    pub terminal_emulator: TerminalEmulator,
    pub ssh_terminal: Option<SshTerminal>,
}

impl Default for LsyShell {
    fn default() -> Self {
        Self {
            connections: load_connections().unwrap_or_default(),
            selected_connection: None,
            new_dialog: NewConnectionDialog::default(),
            session: None,
            terminal_output: String::new(),
            force_cursor_to_end: true,
            pending_input: Some(String::new()),
            command_history: Vec::new(),
            history_index: 0,
            pending_commands: Vec::new(),
            deferred_actions: Vec::new(),
            temp_connection: Some(SshConnection::default()),
            connection_states: HashMap::new(),
            terminal_emulator: TerminalEmulator::new(),
            ssh_terminal: None,
        }
    }
}

// 在LsyShell实现中添加连接管理方法
impl LsyShell {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        load_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self::default()
    }   

    pub fn save_connections(&self) {
        if let Err(e) = save_connections(&self.connections) {
            eprintln!("保存连接失败: {}", e);
        }
        // self.connection_states.retain(|name, _| self.connections.contains_key(name));
    }

    pub fn add_connection(&mut self) {
        let key = self.new_dialog.connection.name.clone();
        self.connections.insert(key, self.new_dialog.connection.clone());
        self.save_connections();
    }

    pub fn show_connection_dialog(&mut self) {
        self.new_dialog.show = true;
    }
    pub fn connect(&mut self, conn_name: &str) {
        if let Some(conn) = self.connections.get(conn_name) {
            match SshTerminal::new(conn) {
                Ok(mut ssh_terminal) => {
                    ssh_terminal.spawn_io_threads();
                    self.ssh_terminal = Some(ssh_terminal);
                    self.terminal_emulator.terminal_output.push_str(&format!("成功连接到 {}\n", conn_name));
                }
                Err(e) => {
                    self.terminal_emulator.terminal_output.push_str(&format!("连接失败: {}\n", e));
                }
            }
        }
    }
    // pub fn connect(&mut self, conn_name: &str) {
    //     if let Some(conn) = self.connections.get(conn_name) {
    //         match connect_ssh(conn) {
    //             Ok(session) => {
    //                 self.session = Some(session);
    //                 self.terminal_output.push_str(&format!("成功连接到 {}\n", conn_name));
    //                 // 初始化终端提示符
    //                 self.send_command("echo 'Welcome to LSYShell'");
                  
    //             }
    //             Err(e) => {
    //                 self.terminal_output.push_str(&format!("连接失败: {}\n", e));
    //             }
    //         }
    //     }
    // }

    pub fn get_connection_state(&mut self, name: &str) -> &mut ConnectionState {
        self.connection_states.entry(name.to_string()).or_default()
    }
    

    
    pub fn start_editing(&mut self, name: &str) {
        if let Some(conn) = self.connections.get(name) {
            self.new_dialog.connection = conn.clone();
            self.new_dialog.show = true;
        }
    }
    
    pub fn mark_for_deletion(&mut self, conn_name: &str) {
        let d_conn = self.connections.get(conn_name).unwrap();
        self.deferred_actions.push(DeferredAction::DeleteConnection(d_conn.clone()));
    }

    pub fn handle_terminal_output(&mut self) {
        // 处理终端的显示字符串内容，最多可以1万行
        // 按换行符分割字符串为行集合
        let lines: Vec<&str> = self.terminal_output.split('\n').collect();

        // 计算起始行索引（最多保留最后1万行）
        let start_index = if lines.len() > 10_000 {
            lines.len() - 10_000
        } else {
            0
        };
        // 重新组合为字符串（保留换行符）
        self.terminal_output = lines[start_index..].join("\n");
    }
    pub fn send_empty_command(&mut self) {
        self.terminal_output.push_str("$ ");
        self.force_cursor_to_end = true;
        self.handle_terminal_output();
    }

  
    // 新增命令执行方法
    pub fn send_command(&mut self, command: &str) {
        if let Some(session) = &self.session {
            // 记录命令历史
            self.command_history.push(command.to_string());
            self.history_index = self.command_history.len();
            
            // 创建SSH通道
            let mut channel = session.channel_session().expect("create channel error");
            channel.exec(command).expect("exec command error");
            
            // 读取命令输出
            let mut output = String::new();
            channel.read_to_string(&mut output).unwrap();
            channel.close().unwrap();
            
            // 显示在终端界面
            self.terminal_output.push_str(&format!("{}\n", output));
            self.terminal_output.push_str("$ ");
        } else {
            self.terminal_output.push_str("未建立有效连接\n");
        }
        self.force_cursor_to_end = true;
        self.handle_terminal_output();
    }
}

impl LsyShell {
    pub fn show_context_menu(
        &mut self,
        ctx: &egui::Context,
        connection_name: &str,
        pos: egui::Pos2,
        menu_flag: &mut Option<egui::Pos2>,
    ) {
        let _layer_id = egui::LayerId::new(egui::Order::Foreground, "connection_context_menu".into());
        let response = egui::Area::new(egui::Id::new(connection_name))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::menu(ui.style()).show(ui, |ui| {
                    ui.set_min_width(120.0);

                    // 连接操作
                    if ui.button("🔄 连接").clicked() {
                        self.selected_connection = Some(connection_name.to_string());
                        self.connect(connection_name);
                        *menu_flag = None;
                    }

                    // 编辑操作
                    if ui.button("✏️ 编辑").clicked() {
                        if let Some(conn) = self.connections.get(connection_name) {
                            self.new_dialog.connection = conn.clone();
                            self.new_dialog.show = true;
                        }
                        *menu_flag = None;
                    }

                    // 删除操作
                    if ui.button("🗑️ 删除").clicked() {
                        if let Some(conn) = self.connections.get(connection_name) {
                            self.deferred_actions.push(DeferredAction::DeleteConnection(
                                conn.clone()
                            ));
                        }
                        *menu_flag = None;
                    }
                });
            })
            .response;

        // 点击外部关闭菜单
        if response.clicked_elsewhere() {
            *menu_flag = None;
        }
    }
}

pub struct SshTerminal {
   pub session: Session,
   pub channel: Channel,
   pub tx: Sender<Vec<u8>>,
   pub rx: Receiver<Vec<u8>>,
}

impl SshTerminal {
    pub fn new(conn: &SshConnection) -> Result<Self, LsyError> {
        let tcp = TcpStream::connect((&*conn.host, conn.port))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        match &conn.auth_method {
            AuthMethod::Password(password) => {
                session.userauth_password(&conn.username, password)?;
            }
            AuthMethod::KeyFile(path) => {
                session.userauth_pubkey_file(&conn.username, None, Path::new(path), None)?;
            }
        }

        let mut s_channel = session.channel_session()?;
        s_channel.request_pty("xterm-256color", None, None)?;
        s_channel.shell()?;

        let (tx, rx) = unbounded();  // 使用 crossbeam-channel

        Ok(Self {
            session,
            channel: s_channel,
            tx,
            rx: rx,
        })
    }

    pub fn spawn_io_threads(&mut self) {
        let mut channel = self.channel.clone();
        let tx = self.tx.clone();
       
   
        // 输出线程
        thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                match channel.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        tx.send(buf[..n].to_vec()).unwrap();
                    }
                    _ => break,
                }
            }
        });

       // 输入线程
       let rx = self.rx.clone();
       let mut channel = self.channel.clone();
       thread::spawn(move || {
        while let Ok(data) = rx.recv() {
            channel.write_all(&data).unwrap();
        }
    });
    }
}

// SSH 连接函数
// fn connect_ssh(conn: &SshConnection) -> Result<Session, LsyError> {
//     let tcp = TcpStream::connect((&*conn.host, conn.port))?;
//     let mut session = Session::new()?;
//     session.set_tcp_stream(tcp);
//     session.handshake()?;

//     // 创建交互式通道
//     // let mut channel = session.channel_session()?;
//     // channel.request_pty("xterm", None, Some((80, 24, 0, 0)))?;
//     // channel.shell()?;
    

//     match &conn.auth_method {
//         AuthMethod::Password(password) => {
//             session.userauth_password(&conn.username, password)?;
//         }
//         AuthMethod::KeyFile(path) => {
//             session.userauth_pubkey_file(&conn.username, None, Path::new(path), None)?;
//         }
//     }

//     Ok(session)
// }

const CONNECTION_PATH: &str = "./config/connections.json";

// 配置文件保存/加载函数（需要实现serde的序列化）
fn save_connections(connections: &HashMap<String, SshConnection>) -> std::io::Result<()> {
    let data = serde_json::to_string(connections)?;
    std::fs::write(CONNECTION_PATH, data)?;
    Ok(())
}

fn load_connections() -> std::io::Result<HashMap<String, SshConnection>> {
    let data = std::fs::read_to_string(CONNECTION_PATH)?;
    Ok(serde_json::from_str(&data)?)
}

