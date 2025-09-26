#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod downloader;
mod network;
mod plugins;
mod ui;
mod utils;
mod loading;
mod mode;
mod source_selector;

use eframe::egui;
use std::env;
use mode::PluginMode;

#[cfg(target_os = "windows")]
fn request_admin() -> bool {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use winapi::um::processthreadsapi::GetCurrentProcess;
    use winapi::um::processthreadsapi::OpenProcessToken;
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
    use winapi::um::handleapi::CloseHandle;
    use std::ptr;
    use std::mem;

    unsafe {
        let mut is_elevated = false;
        let process = GetCurrentProcess();
        let mut token = ptr::null_mut();
        
        if OpenProcessToken(process, TOKEN_QUERY, &mut token) != 0 {
            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut size = 0;
            
            if GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut size,
            ) != 0 {
                is_elevated = elevation.TokenIsElevated != 0;
            }
            
            CloseHandle(token);
        }
        
        if !is_elevated {
            let exe = env::current_exe().unwrap();
            let args: Vec<String> = env::args().skip(1).collect();
            
            let result = Command::new("cmd")
                .arg("/c")
                .arg("start")
                .raw_arg(format!("runas /user:Administrator \"{}\" {}", exe.display(), args.join(" ")))
                .spawn();
                
            if result.is_ok() {
                std::process::exit(0);
            }
        }
        
        is_elevated
    }
}

fn main() -> eframe::Result<()> {
    // 请求管理员权限
    #[cfg(target_os = "windows")]
    {
        request_admin();
    }
    
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    let mode = if args.len() > 1 {
        match args[1].as_str() {
            "--hpm" => PluginMode::HotPE,
            "--edgeless" => PluginMode::Edgeless,
            "--select" => PluginMode::Select,
            _ => PluginMode::CloudPE,
        }
    } else {
        PluginMode::CloudPE
    };
    
    // 初始化运行时
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    
    // 设置图标
    let icon_bytes = include_bytes!("../assets/icon.png");
    let icon = eframe::icon_data::from_png_bytes(icon_bytes).unwrap();
    
    // 根据模式设置窗口标题
    let window_title = match mode {
        PluginMode::CloudPE => "Cloud-PE 插件市场",
        PluginMode::HotPE => "HotPE 模块下载",
        PluginMode::Edgeless => "Edgeless 插件下载",
        PluginMode::Select => "选择插件源",
    };
    
    // 根据模式设置窗口大小
    let window_size = if mode == PluginMode::Select {
        [400.0, 300.0]
    } else {
        [1024.0, 630.0]
    };
    
    // 配置窗口选项
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(window_size)
            .with_min_inner_size(if mode == PluginMode::Select { [400.0, 300.0] } else { [800.0, 600.0] })
            .with_icon(icon)
            .with_resizable(mode != PluginMode::Select),
        centered: true,
        ..Default::default()
    };
    
    // 启动应用
    eframe::run_native(
        window_title,
        native_options,
        Box::new(move |cc| {
            // 加载自定义字体
            setup_custom_fonts(&cc.egui_ctx);
            
            if mode == PluginMode::Select {
                Ok(Box::new(source_selector::SourceSelector::new(cc)))
            } else {
                Ok(Box::new(loading::LoadingScreen::new(cc, rt, mode)))
            }
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 加载中文字体
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/NotoSansSC.ttf"))
    );
    
    // 设置字体优先级
    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());
    
    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("noto_sans_sc".to_owned());
    
    ctx.set_fonts(fonts);
}