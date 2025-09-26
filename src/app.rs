use crate::config::{AppConfig, ColorMode};
use crate::plugins::PluginManager;
use crate::ui::{PluginsMarketPage, PluginsManagePage, SettingsPage};
use crate::utils::BootDriveManager;
use crate::mode::PluginMode;
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::runtime::Runtime;

#[cfg(target_os = "windows")]
use winapi::um::dwmapi::DwmSetWindowAttribute;
#[cfg(target_os = "windows")]
use winapi::um::winuser::GetActiveWindow;
#[cfg(target_os = "windows")]
use std::mem;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    PluginMarket,
    PluginManage,
    Settings,
}

pub struct CloudPEApp {
    config: Arc<RwLock<AppConfig>>,
    current_page: Page,
    plugin_manager: Arc<RwLock<PluginManager>>,
    boot_drive_manager: Arc<RwLock<BootDriveManager>>,
    runtime: Arc<Runtime>,
    mode: PluginMode,
    
    // 页面
    market_page: PluginsMarketPage,
    manage_page: PluginsManagePage,
    settings_page: SettingsPage,
    
    // 状态
    show_boot_drive_dialog: bool,
    selected_boot_drive: Option<String>,
    save_as_default: bool,
    is_first_launch: bool,
}

impl CloudPEApp {
    pub fn new(cc: &eframe::CreationContext<'_>, runtime: Arc<Runtime>, mode: PluginMode) -> Self {
        // 加载配置
        let mut config = AppConfig::load().unwrap_or_default();
        
        // 应用主题
        apply_theme(&cc.egui_ctx, &config.color_mode);
        
        // 初始化管理器
        let boot_drive_manager = Arc::new(RwLock::new(BootDriveManager::new(mode)));
        let plugin_manager = Arc::new(RwLock::new(PluginManager::new(mode)));
        
        // 检查启动盘
        let boot_drives = boot_drive_manager.read().scan_boot_drives();
        let is_first_launch = boot_drives.len() > 1 && config.default_boot_drive.is_none();
        
        // 设置当前启动盘
        if !is_first_launch {
            if let Some(default) = &config.default_boot_drive {
                boot_drive_manager.write().set_current_drive(default.clone());
            } else if boot_drives.len() == 1 {
                boot_drive_manager.write().set_current_drive(boot_drives[0].letter.clone());
                config.default_boot_drive = Some(boot_drives[0].letter.clone());
                config.save().ok();
            }
        }
        
        let config = Arc::new(RwLock::new(config));
        
        // 创建页面
        let market_page = PluginsMarketPage::new(
            plugin_manager.clone(),
            config.clone(),
            runtime.clone(),
            boot_drive_manager.clone(),
            mode,
        );
        let manage_page = PluginsManagePage::new(
            plugin_manager.clone(),
            boot_drive_manager.clone(),
            mode,
        );
        let settings_page = SettingsPage::new(
            config.clone(),
            boot_drive_manager.clone(),
            mode,
        );
        
        Self {
            config,
            current_page: Page::PluginMarket,
            plugin_manager,
            boot_drive_manager,
            runtime,
            mode,
            market_page,
            manage_page,
            settings_page,
            show_boot_drive_dialog: is_first_launch,
            selected_boot_drive: None,
            save_as_default: false,
            is_first_launch,
        }
    }
}

impl eframe::App for CloudPEApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 显示启动盘选择对话框
        if self.show_boot_drive_dialog {
            self.show_boot_drive_selection_dialog(ctx);
            return;
        }
        
        // 左侧导航面板
        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    // 使用 set_min_width 确保标题不换行
                    ui.set_min_width(155.0);
                    ui.label(egui::RichText::new(self.mode.get_title()).heading());
                    ui.separator();
                    
                    if ui.selectable_label(self.current_page == Page::PluginMarket, self.mode.get_plugin_market_name()).clicked() {
                        self.current_page = Page::PluginMarket;
                    }
                    
                    if ui.selectable_label(self.current_page == Page::PluginManage, self.mode.get_plugin_manage_name()).clicked() {
                        self.current_page = Page::PluginManage;
                    }
                    
                    if ui.selectable_label(self.current_page == Page::Settings, "设置").clicked() {
                        self.current_page = Page::Settings;
                    }
                });
            });
        
        // 主内容区
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::PluginMarket => self.market_page.show(ui, ctx),
                Page::PluginManage => self.manage_page.show(ui, ctx),
                Page::Settings => self.settings_page.show(ui, ctx),
            }
        });
        
        // 检查主题变化
        let config = self.config.read();
        apply_theme(ctx, &config.color_mode);
    }
}

impl CloudPEApp {
    fn show_boot_drive_selection_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("选择启动盘")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("检测到多个启动盘，请选择要使用的启动盘：");
                ui.separator();
                
                let boot_drives = self.boot_drive_manager.read().get_all_drives();
                
                egui::ComboBox::from_label("启动盘")
                    .selected_text(self.selected_boot_drive.as_deref().unwrap_or("请选择"))
                    .show_ui(ui, |ui| {
                        for drive in &boot_drives {
                            ui.selectable_value(
                                &mut self.selected_boot_drive,
                                Some(drive.letter.clone()),
                                &drive.letter,
                            );
                        }
                    });
                
                ui.checkbox(&mut self.save_as_default, "把这项选择设为默认值");
                
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() && self.selected_boot_drive.is_some() {
                        if let Some(drive) = &self.selected_boot_drive {
                            self.boot_drive_manager.write().set_current_drive(drive.clone());
                            
                            if self.save_as_default {
                                let mut config = self.config.write();
                                config.default_boot_drive = Some(drive.clone());
                                config.save().ok();
                            }
                            
                            self.show_boot_drive_dialog = false;
                        }
                    }
                });
            });
    }
}

fn apply_theme(ctx: &egui::Context, mode: &ColorMode) {
    let is_dark = match mode {
        ColorMode::System => {
            dark_light::detect() == dark_light::Mode::Dark
        }
        ColorMode::Light => false,
        ColorMode::Dark => true,
    };
    
    let visuals = if is_dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    
    ctx.set_visuals(visuals);
    
    // 在Windows上设置DWM属性
    #[cfg(target_os = "windows")]
    unsafe {
        set_dwm_theme(mode, is_dark);
    }
}

#[cfg(target_os = "windows")]
unsafe fn set_dwm_theme(mode: &ColorMode, is_dark: bool) {
    let hwnd = GetActiveWindow();
    if !hwnd.is_null() {
        let dark_mode = match mode {
            ColorMode::System => {
                // 跟随系统模式，使用系统默认值
                if dark_light::detect() == dark_light::Mode::Dark { 1i32 } else { 0i32 }
            }
            _ => {
                // 手动设置的模式
                if is_dark { 1i32 } else { 0i32 }
            }
        };
        
        DwmSetWindowAttribute(
            hwnd as _,
            20, // DWMWA_USE_IMMERSIVE_DARK_MODE
            &dark_mode as *const _ as *mut _,
            mem::size_of::<i32>() as u32,
        );
    }
}