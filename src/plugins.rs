use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;
use std::collections::HashSet;
use crate::mode::PluginMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub size: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub describe: String,
    #[serde(default)]
    pub file: String,
    pub link: String,
}

impl Plugin {
    // 生成唯一标识，用于去重
    fn get_unique_key(&self) -> String {
        format!("{}_{}_{}_{}", self.name, self.version, self.author, self.size)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCategory {
    pub class: String,
    #[serde(default)]
    pub icon: Option<String>,
    pub list: Vec<Plugin>,
}

// Cloud-PE响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudPEResponse {
    pub code: i32,
    pub message: String,
    pub data: Vec<PluginCategory>,
}

// HotPE响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotPEResponse {
    pub state: String,
    pub data: Vec<HotPECategory>,
}

// HotPE分类格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotPECategory {
    pub class: String,
    #[serde(default)]
    pub icon: Option<String>,
    pub list: Vec<HotPEPlugin>,
}

// HotPE插件格式 - 修复：modified可能是整数或字符串
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotPEPlugin {
    pub name: String,
    pub size: serde_json::Value,  // 可能是整数或字符串
    #[serde(deserialize_with = "deserialize_modified")]
    pub modified: String,
    pub link: String,
}

// 自定义反序列化函数，处理modified字段可能是整数的情况
fn deserialize_modified<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => {
            // 如果是Unix时间戳，转换为字符串
            if let Some(timestamp) = n.as_i64() {
                Ok(format_timestamp(timestamp))
            } else {
                Ok(n.to_string())
            }
        }
        _ => Err(D::Error::custom("expected string or number for modified field")),
    }
}

fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc};
    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        timestamp.to_string()
    }
}

pub struct PluginManager {
    pub categories: Vec<PluginCategory>,
    enabled_plugins: Vec<Plugin>,
    disabled_plugins: Vec<Plugin>,
    mode: PluginMode,
}

impl PluginManager {
    pub fn new(mode: PluginMode) -> Self {
        Self {
            categories: Vec::new(),
            enabled_plugins: Vec::new(),
            disabled_plugins: Vec::new(),
            mode,
        }
    }
    
    // 静态异步方法，不需要 self
    pub async fn fetch_plugins_async(mode: PluginMode) -> Result<Vec<PluginCategory>> {
        let client = reqwest::Client::new();
        let response = client
            .get(mode.get_api_url())
            .send()
            .await?;
        
        let text = response.text().await?;
        
        match mode {
            PluginMode::CloudPE | PluginMode::Edgeless => {
                let mut plugins_response: CloudPEResponse = serde_json::from_str(&text)?;
                
                if plugins_response.code == 200 {
                    // 对每个分类的插件进行去重
                    for category in &mut plugins_response.data {
                        let mut seen = HashSet::new();
                        let mut unique_plugins = Vec::new();
                        
                        for plugin in &category.list {
                            let key = plugin.get_unique_key();
                            if seen.insert(key) {
                                unique_plugins.push(plugin.clone());
                            }
                        }
                        
                        category.list = unique_plugins;
                    }
                    
                    Ok(plugins_response.data)
                } else {
                    anyhow::bail!("获取插件列表失败: {}", plugins_response.message)
                }
            }
            PluginMode::HotPE => {
                // 先尝试解析，如果失败打印原始响应以便调试
                let hotpe_response: HotPEResponse = match serde_json::from_str(&text) {
                    Ok(resp) => resp,
                    Err(e) => {
                        eprintln!("解析HotPE响应失败: {}", e);
                        eprintln!("原始响应: {}", text);
                        return Err(anyhow::anyhow!("解析HotPE响应失败: {}", e));
                    }
                };
                
                if hotpe_response.state == "success" {
                    let mut categories = Vec::new();
                    
                    for hotpe_category in hotpe_response.data {
                        let mut plugins = Vec::new();
                        
                        // 转换HotPE插件格式到通用格式
                        for hotpe_plugin in hotpe_category.list {
                            // 解析文件名
                            let file_name = hotpe_plugin.name.clone();
                            let parts: Vec<&str> = file_name.trim_end_matches(".HPM").split('_').collect();
                            
                            let (name, author, version, describe) = if parts.len() >= 4 {
                                (parts[0].to_string(), parts[1].to_string(), parts[2].to_string(), parts[3..].join("_"))
                            } else if parts.len() == 3 {
                                (parts[0].to_string(), parts[1].to_string(), parts[2].to_string(), String::new())
                            } else {
                                (file_name.clone(), String::new(), String::new(), String::new())
                            };
                            
                            // 处理size字段，可能是整数或字符串
                            let size_str = match hotpe_plugin.size {
                                serde_json::Value::Number(n) => {
                                    if let Some(size) = n.as_i64() {
                                        format_file_size(size)
                                    } else if let Some(size) = n.as_f64() {
                                        format_file_size(size as i64)
                                    } else {
                                        "未知大小".to_string()
                                    }
                                }
                                serde_json::Value::String(s) => s,
                                _ => "未知大小".to_string(),
                            };
                            
                            plugins.push(Plugin {
                                name,
                                size: size_str,
                                version,
                                author,
                                describe,
                                file: hotpe_plugin.name,
                                link: hotpe_plugin.link,
                            });
                        }
                        
                        categories.push(PluginCategory {
                            class: hotpe_category.class,
                            icon: hotpe_category.icon,
                            list: plugins,
                        });
                    }
                    
                    Ok(categories)
                } else {
                    anyhow::bail!("获取HotPE模块列表失败")
                }
            }
            _ => anyhow::bail!("不支持的模式"),
        }
    }
    
    pub fn get_categories(&self) -> &Vec<PluginCategory> {
        &self.categories
    }
    
    pub fn search_plugins(&self, keyword: &str) -> Vec<Plugin> {
        let keyword = keyword.to_lowercase();
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        
        for category in &self.categories {
            for plugin in &category.list {
                let search_text = format!("{} {} {} {}", 
                    plugin.name, plugin.author, plugin.describe, plugin.version).to_lowercase();
                    
                if search_text.contains(&keyword) {
                    // 搜索结果也要去重
                    let key = plugin.get_unique_key();
                    if seen.insert(key) {
                        results.push(plugin.clone());
                    }
                }
            }
        }
        
        results
    }
    
    pub fn load_local_plugins(&mut self, drive_letter: &str) -> Result<()> {
        let plugin_dir = format!("{}\\{}", drive_letter, self.mode.get_plugin_folder());
        let dir_path = Path::new(&plugin_dir);
        
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)?;
        }
        
        self.enabled_plugins.clear();
        self.disabled_plugins.clear();
        
        let mut seen_enabled = HashSet::new();
        let mut seen_disabled = HashSet::new();
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    
                    let enabled_ext = self.mode.get_enabled_extension().to_lowercase();
                    let disabled_ext = self.mode.get_disabled_extension().to_lowercase();
                    
                    // 处理不同模式的文件格式
                    let is_enabled = match self.mode {
                        PluginMode::HotPE => {
                            // HotPE: .HPM 启用，.hpm.off 禁用
                            ext == "hpm" && !file_name.ends_with(".hpm.off")
                        }
                        _ => ext == enabled_ext,
                    };
                    
                    let is_disabled = match self.mode {
                        PluginMode::HotPE => file_name.ends_with(".hpm.off"),
                        _ => ext == disabled_ext,
                    };
                    
                    if is_enabled || is_disabled {
                        if let Some(plugin) = self.parse_plugin_file(&path) {
                            let key = plugin.get_unique_key();
                            
                            if is_enabled {
                                if seen_enabled.insert(key) {
                                    self.enabled_plugins.push(plugin);
                                }
                            } else {
                                if seen_disabled.insert(key) {
                                    self.disabled_plugins.push(plugin);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_plugin_file(&self, path: &Path) -> Option<Plugin> {
        let file_name = path.file_name()?.to_string_lossy().to_string();
        
        match self.mode {
            PluginMode::CloudPE => {
                // Cloud-PE格式: name_version_author_describe.ce
                let parts: Vec<&str> = file_name.split('_').collect();
                
                if parts.len() >= 4 {
                    let name = parts[0].to_string();
                    let version = parts[1].to_string();
                    let author = parts[2].to_string();
                    
                    let describe_with_ext = parts[3..].join("_");
                    let describe = describe_with_ext
                        .strip_suffix(".ce")
                        .or_else(|| describe_with_ext.strip_suffix(".CBK"))
                        .unwrap_or(&describe_with_ext)
                        .to_string();
                    
                    let metadata = fs::metadata(path).ok()?;
                    let size = format!("{:.2} MB", metadata.len() as f64 / 1024.0 / 1024.0);
                    
                    Some(Plugin {
                        name,
                        size,
                        version,
                        author,
                        describe,
                        file: file_name,
                        link: String::new(),
                    })
                } else {
                    None
                }
            }
            PluginMode::HotPE => {
                // HotPE格式: 模块名称_模块作者_模块版本_模块介绍.HPM
                let base_name = file_name
                    .strip_suffix(".HPM")
                    .or_else(|| file_name.strip_suffix(".hpm.off"))
                    .unwrap_or(&file_name);
                    
                let parts: Vec<&str> = base_name.split('_').collect();
                
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    let author = parts[1].to_string();
                    let version = parts[2].to_string();
                    let describe = if parts.len() > 3 {
                        parts[3..].join("_")
                    } else {
                        String::new()
                    };
                    
                    let metadata = fs::metadata(path).ok()?;
                    let size = format!("{:.2} MB", metadata.len() as f64 / 1024.0 / 1024.0);
                    
                    Some(Plugin {
                        name,
                        size,
                        version,
                        author,
                        describe,
                        file: file_name,
                        link: String::new(),
                    })
                } else {
                    None
                }
            }
            PluginMode::Edgeless => {
                // Edgeless格式: 插件名称_插件版本_插件作者.7z
                let base_name = file_name
                    .strip_suffix(".7z")
                    .or_else(|| file_name.strip_suffix(".7zf"))
                    .unwrap_or(&file_name);
                    
                let parts: Vec<&str> = base_name.split('_').collect();
                
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    let version = parts[1].to_string();
                    let author = parts[2..].join("_");
                    
                    let metadata = fs::metadata(path).ok()?;
                    let size = format!("{:.2} MB", metadata.len() as f64 / 1024.0 / 1024.0);
                    
                    Some(Plugin {
                        name,
                        size,
                        version,
                        author,
                        describe: String::new(), // Edgeless没有描述
                        file: file_name,
                        link: String::new(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    pub fn enable_plugin(&mut self, drive_letter: &str, file_name: &str) -> Result<()> {
        let plugin_dir = format!("{}\\{}", drive_letter, self.mode.get_plugin_folder());
        let file_path = Path::new(&plugin_dir).join(file_name);
        
        if !file_path.exists() {
            anyhow::bail!("文件不存在");
        }
        
        let new_file_name = match self.mode {
            PluginMode::CloudPE => file_name.replace(".CBK", ".ce"),
            PluginMode::HotPE => file_name.replace(".hpm.off", ".HPM"),
            PluginMode::Edgeless => file_name.replace(".7zf", ".7z"),
            _ => return Ok(()),
        };
        
        let new_file_path = Path::new(&plugin_dir).join(&new_file_name);
        
        fs::rename(&file_path, &new_file_path)?;
        self.load_local_plugins(drive_letter)?;
        
        Ok(())
    }
    
    pub fn disable_plugin(&mut self, drive_letter: &str, file_name: &str) -> Result<()> {
        let plugin_dir = format!("{}\\{}", drive_letter, self.mode.get_plugin_folder());
        let file_path = Path::new(&plugin_dir).join(file_name);
        
        if !file_path.exists() {
            anyhow::bail!("文件不存在");
        }
        
        let new_file_name = match self.mode {
            PluginMode::CloudPE => file_name.replace(".ce", ".CBK"),
            PluginMode::HotPE => {
                if file_name.ends_with(".HPM") {
                    file_name.replace(".HPM", ".hpm.off")
                } else {
                    format!("{}.off", file_name)
                }
            }
            PluginMode::Edgeless => file_name.replace(".7z", ".7zf"),
            _ => return Ok(()),
        };
        
        let new_file_path = Path::new(&plugin_dir).join(&new_file_name);
        
        fs::rename(&file_path, &new_file_path)?;
        self.load_local_plugins(drive_letter)?;
        
        Ok(())
    }
    
    pub fn get_enabled_plugins(&self) -> &Vec<Plugin> {
        &self.enabled_plugins
    }
    
    pub fn get_disabled_plugins(&self) -> &Vec<Plugin> {
        &self.disabled_plugins
    }
}

fn format_file_size(size: i64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)
    } else {
        format!("{:.2} GB", size as f64 / 1024.0 / 1024.0 / 1024.0)
    }
}