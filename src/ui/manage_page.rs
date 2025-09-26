use crate::plugins::{Plugin, PluginManager};
use crate::utils::BootDriveManager;
use crate::mode::PluginMode;
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct PluginsManagePage {
    plugin_manager: Arc<RwLock<PluginManager>>,
    boot_drive_manager: Arc<RwLock<BootDriveManager>>,
    mode: PluginMode,
}

impl PluginsManagePage {
    pub fn new(
        plugin_manager: Arc<RwLock<PluginManager>>,
        boot_drive_manager: Arc<RwLock<BootDriveManager>>,
        mode: PluginMode,
    ) -> Self {
        Self {
            plugin_manager,
            boot_drive_manager,
            mode,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading(self.mode.get_plugin_manage_name());
        ui.separator();
        
        let current_drive = self.boot_drive_manager.read().get_current_drive();
        
        if let Some(drive) = current_drive {
            // 加载本地插件
            let _ = self.plugin_manager.write().load_local_plugins(&drive);
            
            let enabled_label = match self.mode {
                PluginMode::HotPE => "已启用模块",
                _ => "已启用插件",
            };
            
            let disabled_label = match self.mode {
                PluginMode::HotPE => "已禁用模块",
                _ => "已禁用插件",
            };
            
            ui.collapsing(enabled_label, |ui| {
                let enabled_plugins = self.plugin_manager.read().get_enabled_plugins().clone();
                
                if enabled_plugins.is_empty() {
                    let empty_text = match self.mode {
                        PluginMode::HotPE => "暂无已启用的模块",
                        _ => "暂无已启用的插件",
                    };
                    ui.label(empty_text);
                } else {
                    for plugin in enabled_plugins {
                        self.show_plugin_item(ui, &plugin, true, &drive);
                    }
                }
            });
            
            ui.collapsing(disabled_label, |ui| {
                let disabled_plugins = self.plugin_manager.read().get_disabled_plugins().clone();
                
                if disabled_plugins.is_empty() {
                    let empty_text = match self.mode {
                        PluginMode::HotPE => "暂无已禁用的模块",
                        _ => "暂无已禁用的插件",
                    };
                    ui.label(empty_text);
                } else {
                    for plugin in disabled_plugins {
                        self.show_plugin_item(ui, &plugin, false, &drive);
                    }
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("请先选择或安装启动盘");
            });
        }
    }
    
    fn show_plugin_item(&mut self, ui: &mut egui::Ui, plugin: &Plugin, is_enabled: bool, drive: &str) {
        egui::Frame::default()
            .fill(ui.style().visuals.window_fill())
            .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
            .inner_margin(10.0)
            .outer_margin(5.0)
            .rounding(5.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&plugin.name).strong());
                        
                        // Edgeless模式不显示描述
                        if self.mode != PluginMode::Edgeless && !plugin.describe.is_empty() {
                            ui.label(&plugin.describe);
                        }
                        
                        ui.horizontal(|ui| {
                            ui.label(format!("版本: {}", plugin.version));
                            ui.label(format!("大小: {}", plugin.size));
                            ui.label(format!("作者: {}", plugin.author));
                        });
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_enabled {
                            if ui.button("禁用").clicked() {
                                let _ = self.plugin_manager.write()
                                    .disable_plugin(drive, &plugin.file);
                            }
                        } else {
                            if ui.button("启用").clicked() {
                                let _ = self.plugin_manager.write()
                                    .enable_plugin(drive, &plugin.file);
                            }
                        }
                    });
                });
            });
    }
}