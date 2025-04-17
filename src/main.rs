use eframe::NativeOptions;
use lsyshell::shell::lsyshell::LsyShell;



fn main() {
    env_logger::init();

    let mut native_options = NativeOptions::default();
    native_options.centered = true;

    eframe::run_native("lsyshell", native_options, Box::new(|cc| Ok(Box::new(LsyShell::new(cc))))).unwrap();
}