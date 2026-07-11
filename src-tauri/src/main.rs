#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        // Disable DMABUF / compositing to avoid WebKitWebProcess GPU crashes
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    rdesq::run()
}
