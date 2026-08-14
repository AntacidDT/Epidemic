/// Shared entry point — both desktop and Android call this.
pub fn run() {
    env_logger::init();
    epidemic_render::run();
}
