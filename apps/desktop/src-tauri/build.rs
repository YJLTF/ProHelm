fn main() {
    // tauri-build 只对 tauri.conf.json / capabilities 声明 rerun-if-changed，
    // 图标文件变更不会触发资源重编译（exe 复用旧 .res，桌面图标显示为旧图），
    // 这里显式补充跟踪。
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=icons/32x32.png");
    tauri_build::build()
}
