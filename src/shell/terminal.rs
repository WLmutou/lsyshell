use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ssh2::Session;
use std::io::{Read, Write};


pub struct AnsiProcessor {
    buffer: Vec<u8>,
}

impl AnsiProcessor {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn process(&mut self, byte: u8) -> Option<String> {
        self.buffer.push(byte);
        
        // 检查是否是完整的ANSI序列
        if byte == b'm' || byte == b'h' || byte == b'l' {
            let sequence = String::from_utf8_lossy(&self.buffer).to_string();
            self.buffer.clear();
            
            // 这里可以添加对特定序列的处理逻辑
            // 目前我们只是过滤掉它们
            return None;
        }
        
        // 如果不是ANSI序列的一部分，返回普通字符
        if self.buffer.len() == 1 && self.buffer[0] != 0x1b {
            let ch = self.buffer[0] as char;
            self.buffer.clear();
            return Some(ch.to_string());
        }
        
        None
    }
    
    pub fn flush(&mut self) -> Option<String> {
        if !self.buffer.is_empty() {
            let content = String::from_utf8_lossy(&self.buffer).to_string();
            self.buffer.clear();
            Some(content)
        } else {
            None
        }
    }
}

pub struct TerminalEmulator {

    pub terminal_output: String,
    pub cursor_position: usize,
    pub alternate_screen: bool,
    pub ansi_processor: AnsiProcessor,
}

impl Default for TerminalEmulator {
    fn default() -> Self {
        Self {
            terminal_output: String::new(),
            // content: String::new(),
            cursor_position: 0,
            alternate_screen: false,
            ansi_processor: AnsiProcessor::new(),
        }
    }
}

impl TerminalEmulator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn write(&mut self, data: &[u8]) {
        for &byte in data {
            if let Some(text) = self.ansi_processor.process(byte) {
                self.terminal_output.push_str(&text);
            }
        }
        
        // 刷新处理器中可能剩余的内容
        if let Some(text) = self.ansi_processor.flush() {
            self.terminal_output.push_str(&text);
        }
        
        // 限制内容长度
        if self.terminal_output.len() > 10_000 {
            self.terminal_output = self.terminal_output.split_off(self.terminal_output.len() - 10_000);
        }
    }
    // pub fn write(&mut self, data: &[u8]) {
    //     // 处理终端控制序列
    //     let output = String::from_utf8_lossy(data);
    //     self.terminal_output.push_str(&output);
    //     self.cursor_position = self.terminal_output.len(); 
        
    // }

    pub fn read(&mut self) -> Vec<u8> {
        // 处理用户输入
        // 这里简化处理，实际需要处理键盘事件
        Vec::new()
    }

    pub fn enter_alternate_screen(&mut self) {
        self.alternate_screen = true;
        self.terminal_output.clear();
    }

    pub fn leave_alternate_screen(&mut self) {
        self.alternate_screen = false;
    }

    pub fn get_output(&self) -> &str {
        &self.terminal_output
    }
    pub fn get_content(&self) -> &str {
        &self.terminal_output
    }
    
    pub fn get_content_mut(&mut self) -> &mut String {
        &mut self.terminal_output
    }
}


