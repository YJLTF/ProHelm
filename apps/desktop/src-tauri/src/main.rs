// 屏蔽 release 模式下的控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    prohelm_desktop_lib::run()
}
