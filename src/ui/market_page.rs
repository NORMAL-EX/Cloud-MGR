use crate::plugins::{Plugin, PluginManager};
use crate::config::AppConfig;
use crate::downloader::Downloader;
use crate::utils::BootDriveManager;
use crate::mode::PluginMode;
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::runtime::Runtime;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct DownloadTask {
    plugin_name: String,
    progress: Arc<RwLock<f32>>,
    is_install: bool,
}

pub struct PluginsMarketPage {
    plugin_manager: Arc<RwLock<PluginManager>>,
    config: Arc<RwLock<AppConfig>>,
    runtime: Arc<Runtime>,
    boot_drive_manager: Arc<RwLock<BootDriveManager>>,
    search_text: String,
    selected_category: String,
    last_selected_category: String,
    downloading_tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    is_loading: bool,
    show_search_category: bool,
    mode: PluginMode,
}

impl PluginsMarketPage {
    pub fn new(
        plugin_manager: Arc<RwLock<PluginManager>>,
        config: Arc<RwLock<AppConfig>>,
        runtime: Arc<Runtime>,
        boot_drive_manager: Arc<RwLock<BootDriveManager>>,
        mode: PluginMode,
    ) -> Self {
        let plugin_manager_clone = plugin_manager.clone();
        let runtime_clone = runtime.clone();
        let mode_clone = mode.clone();
        
        let page = Self {
            plugin_manager: plugin_manager.clone(),
            config,
            runtime: runtime.clone(),
            boot_drive_manager,
            search_text: String::new(),
            selected_category: "推荐".to_string(),
            last_selected_category: "推荐".to_string(),
            downloading_tasks: Arc::new(RwLock::new(HashMap::new())),
            is_loading: true,
            show_search_category: false,
            mode,
        };
        
        // 初始加载插件
        runtime_clone.spawn(async move {
            match PluginManager::fetch_plugins_async(mode_clone).await {
                Ok(categories) => {
                    plugin_manager_clone.write().categories = categories;
                }
                Err(e) => {
                    eprintln!("加载插件失败: {}", e);
                }
            }
        });
        
        page
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // 检查加载状态
        if self.is_loading && !self.plugin_manager.read().get_categories().is_empty() {
            self.is_loading = false;
            // 确保有推荐分类
            if !self.plugin_manager.read().get_categories().iter().any(|c| c.class == "推荐") {
                // 如果没有推荐分类，选择第一个分类
                if let Some(first_category) = self.plugin_manager.read().get_categories().first() {
                    self.selected_category = first_category.class.clone();
                    self.last_selected_category = first_category.class.clone();
                }
            }
        }
        
        // 顶部搜索栏
        ui.horizontal(|ui| {
            ui.heading(self.mode.get_plugin_market_name());
            ui.add_space(20.0);
            
            ui.label("搜索：");
            let response = ui.text_edit_singleline(&mut self.search_text);
            
            // 处理搜索文本变化
            if response.changed() {
                if !self.search_text.is_empty() {
                    // 有搜索内容，显示搜索分类
                    if !self.show_search_category {
                        self.show_search_category = true;
                        // 记录当前选中的分类（如果不是搜索分类）
                        if self.selected_category != "搜索" {
                            self.last_selected_category = self.selected_category.clone();
                        }
                        self.selected_category = "搜索".to_string();
                    }
                } else {
                    // 清空搜索内容，隐藏搜索分类
                    if self.show_search_category {
                        self.show_search_category = false;
                        self.selected_category = self.last_selected_category.clone();
                    }
                }
            }
        });
        
        ui.separator();
        
        // 分类选择（多行布局）
        if !self.is_loading {
            let categories = self.plugin_manager.read().get_categories().clone();
            if !categories.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    // 如果显示搜索分类，先添加搜索按钮
                    if self.show_search_category {
                        if ui.selectable_label(self.selected_category == "搜索", "搜索").clicked() {
                            self.selected_category = "搜索".to_string();
                        }
                    }
                    
                    // 显示其他分类
                    for category in &categories {
                        if ui.selectable_label(self.selected_category == category.class, &category.class).clicked() {
                            self.selected_category = category.class.clone();
                            if !self.show_search_category || self.selected_category != "搜索" {
                                self.last_selected_category = category.class.clone();
                            }
                        }
                    }
                });
                ui.separator();
            }
        }
        
        // 插件列表
        egui::ScrollArea::vertical()
            .id_salt("plugin_scroll")
            .show(ui, |ui| {
                if self.is_loading {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        let loading_text = match self.mode {
                            PluginMode::HotPE => "正在加载模块列表...",
                            _ => "正在加载插件列表...",
                        };
                        ui.label(loading_text);
                    });
                } else {
                    let plugins = if self.selected_category == "搜索" && !self.search_text.is_empty() {
                        self.plugin_manager.read().search_plugins(&self.search_text)
                    } else if self.selected_category != "搜索" {
                        self.get_category_plugins()
                    } else {
                        Vec::new()
                    };
                    
                    if plugins.is_empty() {
                        ui.centered_and_justified(|ui| {
                            if self.selected_category == "搜索" {
                                let not_found_text = match self.mode {
                                    PluginMode::HotPE => "未找到相关模块",
                                    _ => "未找到相关插件",
                                };
                                ui.label(not_found_text);
                            } else {
                                let empty_text = match self.mode {
                                    PluginMode::HotPE => "该分类暂无模块",
                                    _ => "该分类暂无插件",
                                };
                                ui.label(empty_text);
                            }
                        });
                    } else {
                        // 再次去重，以防万一
                        let mut seen = HashSet::new();
                        for plugin in plugins {
                            let key = format!("{}_{}_{}_{}", 
                                plugin.name, plugin.version, plugin.author, plugin.size);
                            if seen.insert(key) {
                                self.show_plugin_card(ui, &plugin);
                            }
                        }
                    }
                }
            });
        
        // 定期刷新以更新下载进度
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
    
    fn get_category_plugins(&self) -> Vec<Plugin> {
        let manager = self.plugin_manager.read();
        let categories = manager.get_categories();
        
        categories
            .iter()
            .find(|c| c.class == self.selected_category)
            .map(|c| c.list.clone())
            .unwrap_or_default()
    }
    
    fn show_plugin_card(&mut self, ui: &mut egui::Ui, plugin: &Plugin) {
        egui::Frame::default()
            .fill(ui.style().visuals.window_fill())
            .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
            .inner_margin(10.0)
            .outer_margin(5.0)
            .rounding(5.0)
            .show(ui, |ui| {
                // 响应式布局
                let available_width = ui.available_width();
                
                if available_width > 400.0 {
                    // 宽屏布局：水平排列
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_max_width(available_width - 180.0);
                            ui.label(egui::RichText::new(&plugin.name).strong());
                            
                            // Edgeless模式不显示描述
                            if self.mode != PluginMode::Edgeless && !plugin.describe.is_empty() {
                                ui.label(&plugin.describe);
                            }
                            
                            ui.horizontal_wrapped(|ui| {
                                ui.label(format!("版本: {}", plugin.version));
                                ui.separator();
                                ui.label(format!("大小: {}", plugin.size));
                                ui.separator();
                                ui.label(format!("作者: {}", plugin.author));
                            });
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            self.show_plugin_actions(ui, plugin);
                        });
                    });
                } else {
                    // 窄屏布局：垂直排列
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&plugin.name).strong());
                        
                        // Edgeless模式不显示描述
                        if self.mode != PluginMode::Edgeless && !plugin.describe.is_empty() {
                            ui.label(&plugin.describe);
                        }
                        
                        ui.horizontal_wrapped(|ui| {
                            ui.label(format!("版本: {}", plugin.version));
                            ui.separator();
                            ui.label(format!("大小: {}", plugin.size));
                            ui.separator();
                            ui.label(format!("作者: {}", plugin.author));
                        });
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            self.show_plugin_actions(ui, plugin);
                        });
                    });
                }
            });
    }
    
    fn show_plugin_actions(&mut self, ui: &mut egui::Ui, plugin: &Plugin) {
        let plugin_id_install = format!("{}_{}install", plugin.name, plugin.version);
        let plugin_id_download = format!("{}_{}", plugin.name, plugin.version);
        
        let tasks = self.downloading_tasks.read();
        let is_installing = tasks.contains_key(&plugin_id_install);
        let is_downloading = tasks.contains_key(&plugin_id_download);
        drop(tasks);
        
        // 检查是否有启动盘
        let has_boot_drive = self.boot_drive_manager.read().get_current_drive().is_some();
        
        ui.horizontal(|ui| {
            // 只有有启动盘时才显示安装按钮
            if has_boot_drive {
                if is_installing {
                    ui.add_enabled(false, egui::Button::new("安装中..."));
                    ui.spinner();
                } else {
                    if ui.button("安装").clicked() {
                        self.install_plugin(plugin.clone());
                    }
                }
            }
            
            // 下载按钮始终显示
            if is_downloading {
                ui.add_enabled(false, egui::Button::new("下载中..."));
                ui.spinner();
            } else {
                if ui.button("下载").clicked() {
                    self.download_plugin(plugin.clone());
                }
            }
        });
    }
    
    fn install_plugin(&mut self, plugin: Plugin) {
        let plugin_id = format!("{}_{}install", plugin.name, plugin.version);
        let task = DownloadTask {
            plugin_name: plugin.name.clone(),
            progress: Arc::new(RwLock::new(0.0)),
            is_install: true,
        };
        
        self.downloading_tasks.write().insert(plugin_id.clone(), task.clone());
        
        let downloader = Arc::new(Downloader::new(self.config.read().download_threads));
        let boot_drive = self.boot_drive_manager.read().get_current_drive();
        
        if let Some(drive_letter) = boot_drive {
            // 生成正确的文件名格式
            let filename = self.generate_plugin_filename(&plugin);
            let plugin_name = plugin.name.clone();
            let plugin_url = plugin.link.clone();
            let downloading_tasks = self.downloading_tasks.clone();
            let mode = self.mode.clone();
            
            self.runtime.spawn(async move {
                println!("开始安装插件: {}", plugin_name);
                
                // 使用选择的启动盘路径
                let plugin_dir = format!("{}\\{}", drive_letter, mode.get_plugin_folder());
                
                // 创建目录
                if let Err(e) = std::fs::create_dir_all(&plugin_dir) {
                    eprintln!("创建目录失败: {}", e);
                    downloading_tasks.write().remove(&plugin_id);
                    return;
                }
                
                let extension = mode.get_enabled_extension();
                let install_path = std::path::PathBuf::from(plugin_dir).join(format!("{}.{}", filename, extension));
                
                match downloader.download(&plugin_url, install_path.clone()).await {
                    Ok(_) => {
                        println!("插件 {} 安装成功，路径: {:?}", plugin_name, install_path);
                    }
                    Err(e) => {
                        eprintln!("插件 {} 安装失败: {}", plugin_name, e);
                    }
                }
                
                // 下载完成后移除任务
                downloading_tasks.write().remove(&plugin_id);
            });
        } else {
            // 没有启动盘，移除任务
            self.downloading_tasks.write().remove(&plugin_id);
            eprintln!("没有选择启动盘，无法安装插件");
        }
    }
    
    fn download_plugin(&mut self, plugin: Plugin) {
        use rfd::AsyncFileDialog;
        
        let plugin_id = format!("{}_{}", plugin.name, plugin.version);
        let task = DownloadTask {
            plugin_name: plugin.name.clone(),
            progress: Arc::new(RwLock::new(0.0)),
            is_install: false,
        };
        
        self.downloading_tasks.write().insert(plugin_id.clone(), task.clone());
        
        let config = self.config.clone();
        let downloading_tasks = self.downloading_tasks.clone();
        let runtime = self.runtime.clone();
        
        // 生成正确的文件名格式
        let filename = self.generate_plugin_filename(&plugin);
        let extension = self.mode.get_enabled_extension();
        let full_filename = format!("{}.{}", filename, extension);
        
        let plugin_name = plugin.name.clone();
        let plugin_url = plugin.link.clone();
        
        // 修复：在 spawn 前读取配置，避免跨 await 持有锁
        let default_download_path = config.read().default_download_path.clone();
        
        // 使用异步文件对话框，避免阻塞UI
        runtime.spawn(async move {
            let download_path = if let Some(path) = default_download_path {
                path
            } else {
                match AsyncFileDialog::new()
                    .set_title("选择下载位置")
                    .pick_folder()
                    .await
                {
                    Some(handle) => {
                        let path = handle.path().to_path_buf();
                        let mut config_write = config.write();
                        config_write.default_download_path = Some(path.clone());
                        let _ = config_write.save();
                        path
                    }
                    None => {
                        downloading_tasks.write().remove(&plugin_id);
                        return;
                    }
                }
            };
            
            let downloader = Arc::new(Downloader::new(config.read().download_threads));
            let file_path = download_path.join(full_filename);
            
            println!("开始下载插件 {} 到: {:?}", plugin_name, file_path);
            match downloader.download(&plugin_url, file_path).await {
                Ok(_) => {
                    println!("插件 {} 下载成功", plugin_name);
                }
                Err(e) => {
                    eprintln!("插件 {} 下载失败: {}", plugin_name, e);
                }
            }
            
            // 下载完成后移除任务
            downloading_tasks.write().remove(&plugin_id);
        });
    }
    
    fn generate_plugin_filename(&self, plugin: &Plugin) -> String {
        // 处理特殊字符
        let safe_describe = plugin.describe
            .replace(' ', "_")
            .replace('/', "_")
            .replace('\\', "_")
            .replace(':', "_")
            .replace('*', "_")
            .replace('?', "_")
            .replace('"', "_")
            .replace('<', "_")
            .replace('>', "_")
            .replace('|', "_");
        
        match self.mode {
            PluginMode::CloudPE => {
                format!("{}_{}_{}_{}", plugin.name, plugin.version, plugin.author, safe_describe)
            }
            PluginMode::HotPE => {
                if safe_describe.is_empty() {
                    format!("{}_{}_{}_{}", plugin.name, plugin.author, plugin.version, plugin.name)
                } else {
                    format!("{}_{}_{}_{}", plugin.name, plugin.author, plugin.version, safe_describe)
                }
            }
            PluginMode::Edgeless => {
                format!("{}_{}_{}", plugin.name, plugin.version, plugin.author)
            }
            _ => String::new()
        }
    }
}