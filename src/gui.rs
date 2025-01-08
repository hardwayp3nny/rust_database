use eframe::egui;
use crate::part1::{Database, Table, Row, DataType, Column};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

pub struct DatabaseGui {
    database: Option<Database>,
    is_secure: bool,
    query_input: String,
    output_text: String,
    // 用于创建表的临时状态
    new_table_name: String,
    new_column_name: String,
    new_column_type: String,
    new_column_length: String,
    temp_columns: Vec<Column>,
    // 用于插入数据的临时状态
    selected_table: String,
    insert_values: HashMap<String, String>,
    // 当前视图状态
    current_view: ViewState,
    // 错误信息
    error_message: String,
}

#[derive(PartialEq)]
enum ViewState {
    Main,
    CreateTable,
    InsertData,
    QueryView,
}

impl Default for DatabaseGui {
    fn default() -> Self {
        Self {
            database: None,
            is_secure: true,
            query_input: String::new(),
            output_text: String::new(),
            new_table_name: String::new(),
            new_column_name: String::new(),
            new_column_type: String::from("Int"),
            new_column_length: String::new(),
            temp_columns: Vec::new(),
            selected_table: String::new(),
            insert_values: HashMap::new(),
            current_view: ViewState::Main,
            error_message: String::new(),
        }
    }
}

impl eframe::App for DatabaseGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 顶部菜单栏
            ui.horizontal(|ui| {
                if ui.button("主页").clicked() {
                    self.current_view = ViewState::Main;
                }
                if ui.button("创建表").clicked() {
                    self.current_view = ViewState::CreateTable;
                }
                if ui.button("插入数据").clicked() {
                    self.current_view = ViewState::InsertData;
                }
                if ui.button("SQL查询").clicked() {
                    self.current_view = ViewState::QueryView;
                }
            });

            ui.separator();

            // 如果有错误消息，显示它
            if !self.error_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.error_message);
                if ui.button("清除错误").clicked() {
                    self.error_message.clear();
                }
                ui.separator();
            }

            // 根据当前视图显示不同的内容
            match self.current_view {
                ViewState::Main => self.show_main_view(ui),
                ViewState::CreateTable => self.show_create_table_view(ui),
                ViewState::InsertData => self.show_insert_data_view(ui),
                ViewState::QueryView => self.show_query_view(ui),
            }
        });
    }
}

impl DatabaseGui {
    fn calculate_hash(&self) -> String {
        if let Some(ref db) = self.database {
            if let Ok(json) = serde_json::to_string(db) {
                let mut hasher = Sha256::new();
                hasher.update(json.as_bytes());
                format!("{:x}", hasher.finalize())
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    fn save_database(&mut self) {
        if let Some(ref db) = self.database {
            if let Ok(json) = serde_json::to_string_pretty(db) {
                if let Err(e) = fs::write("database.json", &json) {
                    self.error_message = format!("保存数据库失败: {}", e);
                    return;
                }
                
                let hash = self.calculate_hash();
                if let Err(e) = fs::write("database_hash.txt", &hash) {
                    self.error_message = format!("保存哈希值失败: {}", e);
                }
            }
        }
    }

    fn load_database(&mut self) {
        if Path::new("database.json").exists() {
            match fs::read_to_string("database.json") {
                Ok(json) => {
                    match serde_json::from_str(&json) {
                        Ok(db) => {
                            self.database = Some(db);
                            
                            let current_hash = self.calculate_hash();
                            
                            if Path::new("database_hash.txt").exists() {
                                if let Ok(stored_hash) = fs::read_to_string("database_hash.txt") {
                                    self.is_secure = current_hash == stored_hash;
                                } else {
                                    self.is_secure = false;
                                }
                            } else {
                                if let Err(e) = fs::write("database_hash.txt", &current_hash) {
                                    self.error_message = format!("创建哈希文件失败: {}", e);
                                }
                                self.is_secure = true;
                            }
                            
                            self.output_text = "数据库加载成功！".to_string();
                        },
                        Err(e) => self.error_message = format!("解析数据库失败: {}", e),
                    }
                },
                Err(e) => self.error_message = format!("读取数据库文件失败: {}", e),
            }
        }
    }

    fn show_main_view(&mut self, ui: &mut egui::Ui) {
        if let Some(ref db) = self.database {
            let status_text = if self.is_secure {
                "数据库安全性：正常"
            } else {
                "数据库安全性：不安全"
            };
            ui.colored_label(
                if self.is_secure { egui::Color32::GREEN } else { egui::Color32::RED },
                status_text
            );
            ui.separator();
        }
        
        if self.database.is_none() {
            ui.horizontal(|ui| {
                if ui.button("创建数据库").clicked() {
                    self.database = Some(Database::new("test_db"));
                    self.output_text = "数据库创建成功！".to_string();
                    self.save_database(); // 保存新创建的数据库
                }
                if ui.button("加载数据库").clicked() {
                    self.load_database();
                }
            });
        } else {
            ui.heading("数据库概览");
            if let Some(ref db) = self.database {
                for table in &db.tables {
                    ui.collapsing(&table.name, |ui| {
                        ui.label("列：");
                        for col in &table.columns {
                            ui.label(format!("{}: {:?}", col.name, col.data_type));
                        }
                        ui.label(format!("行数：{}", table.rows.len()));
                        
                        // 顯示表內容
                        ui.separator();
                        ui.label("表内容：");
                        if !table.rows.is_empty() {
                            // 首先顯示列名
                            ui.horizontal(|ui| {
                                for col in &table.columns {
                                    ui.label(&col.name);
                                    ui.add_space(10.0); // 添加一些間距
                                }
                            });
                            
                            // 然後顯示數據行
                            for row in &table.rows {
                                ui.horizontal(|ui| {
                                    for value in &row.values {
                                        ui.label(value.as_ref().unwrap_or(&"NULL".to_string()));
                                        ui.add_space(10.0); // 添加一些間距
                                    }
                                });
                            }
                        } else {
                            ui.label("表中暫無數據");
                        }
                    });
                }
            }
            if ui.button("保存数据库").clicked() {
                self.save_database();
                self.output_text = "数据库保存成功！".to_string();
            }
        }
    }

    fn show_create_table_view(&mut self, ui: &mut egui::Ui) {
        if self.database.is_none() {
            ui.label("请先创建数库！");
            return;
        }

        ui.heading("创建新表");
        ui.horizontal(|ui| {
            ui.label("表名：");
            ui.text_edit_singleline(&mut self.new_table_name);
        });

        ui.group(|ui| {
            ui.label("添加列：");
            ui.horizontal(|ui| {
                ui.label("列名：");
                ui.text_edit_singleline(&mut self.new_column_name);
                ui.label("类型：");
                egui::ComboBox::from_label("")
                    .selected_text(&self.new_column_type)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_column_type, "Int".to_string(), "Int");
                        ui.selectable_value(&mut self.new_column_type, "Char".to_string(), "Char");
                        ui.selectable_value(&mut self.new_column_type, "Bool".to_string(), "Bool");
                        ui.selectable_value(&mut self.new_column_type, "String".to_string(), "String");
                    });

                if self.new_column_type == "Char" || self.new_column_type == "String" {
                    ui.label("长度：");
                    ui.text_edit_singleline(&mut self.new_column_length);
                }

                if ui.button("添加列").clicked() {
                    if !self.new_column_name.is_empty() {
                        let column = match self.new_column_type.as_str() {
                            "Int" => Column {
                                name: self.new_column_name.clone(),
                                data_type: DataType::Int,
                                is_primary_key: false,
                            },
                            "Bool" => Column {
                                name: self.new_column_name.clone(),
                                data_type: DataType::Bool,
                                is_primary_key: false,
                            },
                            "Char" => {
                                if let Ok(len) = self.new_column_length.parse() {
                                    Column {
                                        name: self.new_column_name.clone(),
                                        data_type: DataType::Char(len),
                                        is_primary_key: false,
                                    }
                                } else {
                                    self.error_message = "无效的长度值".to_string();
                                    return;
                                }
                            },
                            "String" => {
                                if let Ok(len) = self.new_column_length.parse() {
                                    Column {
                                        name: self.new_column_name.clone(),
                                        data_type: DataType::String(len),
                                        is_primary_key: false,
                                    }
                                } else {
                                    self.error_message = "无效的长度值".to_string();
                                    return;
                                }
                            },
                            _ => return,
                        };
                        self.temp_columns.push(column);
                        self.new_column_name.clear();
                        self.new_column_length.clear();
                    }
                }
            });
        });

        // 显示当前添加的列
        ui.group(|ui| {
            ui.label("当前列：");
            let mut columns_to_remove = Vec::new(); // 收集要删除的列名
            for (i, col) in self.temp_columns.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}: {:?}", col.name, col.data_type));
                    if ui.button("删除").clicked() {
                        let col_name = col.name.clone();
                        columns_to_remove.push(col_name); // 将要删除的列名添加到列表中
                    }
                });
            }
            // 在循环外进行删除
            for col_name in columns_to_remove {
                self.temp_columns.retain(|col| col.name != col_name);
            }
        });

        if ui.button("创建表").clicked() {
            if let Some(ref mut db) = self.database {
                if !self.new_table_name.is_empty() && !self.temp_columns.is_empty() {
                    let table = Table {
                        name: self.new_table_name.clone(),
                        columns: self.temp_columns.clone(),
                        rows: Vec::new(),
                    };
                    db.create_table(table);
                    self.new_table_name.clear();
                    self.temp_columns.clear();
                    self.output_text = "表创建成功！".to_string();
                    self.save_database(); // 保存更改
                } else {
                    self.error_message = "表名和列不能为空".to_string();
                }
            }
        }
    }

    fn show_insert_data_view(&mut self, ui: &mut egui::Ui) {
        if self.database.is_none() {
            ui.label("请先创建数据库！");
            return;
        }

        ui.heading("插入数据");
        
        if let Some(ref db) = self.database {
            // 选择表
            egui::ComboBox::from_label("选择表")
                .selected_text(&self.selected_table)
                .show_ui(ui, |ui| {
                    for table in &db.tables {
                        ui.selectable_value(&mut self.selected_table, table.name.clone(), &table.name);
                    }
                });

            if let Some(table) = db.tables.iter().find(|t| t.name == self.selected_table) {
                ui.group(|ui| {
                    for col in &table.columns {
                        ui.horizontal(|ui| {
                            ui.label(&col.name);
                            let value = self.insert_values.entry(col.name.clone()).or_insert_with(String::new);
                            ui.text_edit_singleline(value);
                        });
                    }
                });

                if ui.button("插入").clicked() {
                    let mut values = Vec::new();
                    for col in &table.columns {
                        values.push(Some(self.insert_values.get(&col.name).cloned().unwrap_or_default()));
                    }
                    if let Some(ref mut db) = self.database {
                        let row = Row { values };
                        db.insert_row(&self.selected_table, row);
                        self.output_text = "数据插入成功！".to_string();
                        self.insert_values.clear();
                    }
                    self.save_database(); // 保存更改
                }
            }
        }
    }

    fn show_query_view(&mut self, ui: &mut egui::Ui) {
        if self.database.is_none() {
            ui.label("请先创建数据库！");
            return;
        }

        ui.heading("SQL查询");
        
        // SQL输入区域
        ui.group(|ui| {
            ui.label("输入SQL查询");
            ui.text_edit_multiline(&mut self.query_input);
        });

        if ui.button("执行查询").clicked() {
            self.execute_sql_query(ui);
        }

        // 输出区域
        ui.group(|ui| {
            ui.label("查询结果：");
            ui.add(egui::TextEdit::multiline(&mut self.output_text)
                .interactive(false));
        });
    }

    fn execute_sql_query(&mut self, _ui: &mut egui::Ui) {
        if let Some(ref mut db) = self.database {
            let query = self.query_input.trim();
            let query_upper = query.to_uppercase();

            if query_upper.starts_with("SELECT") {
                // 解析 SELECT 語句
                let parts: Vec<&str> = query_upper.split("FROM").collect();
                if parts.len() != 2 {
                    self.error_message = "無效的 SELECT 語句格式".to_string();
                    return;
                }

                let from_part = parts[1].trim();
                
                if from_part.contains("JOIN") {
                    // ... 保持現有的 JOIN 邏輯 ...
                } else if from_part.contains("AND") {
                    // 處理多表並行查詢
                    let table_names: Vec<&str> = from_part
                        .split("AND")
                        .map(|s| s.trim())
                        .collect();

                    let mut output = String::with_capacity(1024);
                    
                    // 為每個表生成查詢結果
                    for table_name in table_names {
                        if let Some(table) = db.tables.iter().find(|t| t.name.to_uppercase() == table_name) {
                            // 添加表名作為標題
                            output.push_str(&format!("\n表 {} 的查詢結果：\n", table_name));
                            
                            // 生成表頭
                            let header = table.columns.iter()
                                .map(|col| format!("{:<20}", col.name))
                                .collect::<Vec<_>>()
                                .join(" | ");
                            output.push_str(&header);
                            output.push('\n');

                            // 添加分隔線
                            let separator = "-".repeat(header.len());
                            output.push_str(&separator);
                            output.push('\n');

                            // 添加數據行
                            for row in &table.rows {
                                let row_str = row.values.iter()
                                    .map(|v| format!("{:<20}", v.as_ref().unwrap_or(&"NULL".to_string())))
                                    .collect::<Vec<_>>()
                                    .join(" | ");
                                output.push_str(&row_str);
                                output.push('\n');
                            }
                            
                            // 添加表間分隔
                            output.push_str("\n");
                        } else {
                            self.error_message = format!("表 '{}' 不存在", table_name);
                            return;
                        }
                    }

                    self.output_text = output;
                } else {
                    // 這裡是單表查詢的邏輯
                    let table_name = from_part.split_whitespace().next().unwrap_or("");
                    if let Some(table) = db.tables.iter().find(|t| t.name.to_uppercase() == table_name) {
                        // 添加表名作為標題
                        self.output_text = format!("\n表 {} 的查詢結果：\n", table_name);
                        
                        // 生成表頭
                        let header = table.columns.iter()
                            .map(|col| format!("{:<20}", col.name))
                            .collect::<Vec<_>>()
                            .join(" | ");
                        self.output_text.push_str(&header);
                        self.output_text.push('\n');

                        // 添加分隔線
                        let separator = "-".repeat(header.len());
                        self.output_text.push_str(&separator);
                        self.output_text.push('\n');

                        // 添加數據行
                        for row in &table.rows {
                            let row_str = row.values.iter()
                                .map(|v| format!("{:<20}", v.as_ref().unwrap_or(&"NULL".to_string())))
                                .collect::<Vec<_>>()
                                .join(" | ");
                            self.output_text.push_str(&row_str);
                            self.output_text.push('\n');
                        }
                    } else {
                        self.error_message = format!("表 '{}' 不存在", table_name);
                    }
                }
            } else if query_upper.starts_with("INSERT INTO") {
                // 解析 INSERT 語句
                let parts: Vec<&str> = query.split("VALUES").collect();
                if parts.len() != 2 {
                    self.error_message = "無效的 INSERT 語句格式".to_string();
                    return;
                }

                // 獲取表名
                let table_name = parts[0]
                    .trim()
                    .strip_prefix("INSERT INTO")
                    .unwrap_or("")
                    .trim();

                // 解析值
                let values_str = parts[1]
                    .trim()
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .trim();

                // 分割值
                let values: Vec<String> = values_str
                    .split(',')
                    .map(|s| s.trim().trim_matches('\'').to_string())
                    .collect();

                // 查找表
                if let Some(table) = db.tables.iter().find(|t| t.name == table_name) {
                    // 檢查值的數量是否匹配列的數量
                    if values.len() != table.columns.len() {
                        self.error_message = format!(
                            "值的數量 ({}) 與列的數量 ({}) 不匹配",
                            values.len(),
                            table.columns.len()
                        );
                        return;
                    }

                    // 創建新行
                    let row = Row {
                        values: values.into_iter().map(Some).collect(),
                    };

                    // 插入數據
                    db.insert_row(table_name, row);
                    self.output_text = "數據插入成功！".to_string();
                    self.save_database(); // 保存更改
                } else {
                    self.error_message = format!("表 '{}' 不存在", table_name);
                }
            } else if query_upper.starts_with("DELETE FROM") {
                // 解析 DELETE 語句
                let parts: Vec<&str> = query[11..].trim().split("WHERE").collect();
                let table_name = parts[0].trim();

                if let Some(table) = db.tables.iter_mut().find(|t| t.name == table_name) {
                    if parts.len() == 1 {
                        // 無 WHERE 子句，刪除所有記錄
                        table.rows.clear();
                        self.output_text = format!("已刪除表 '{}' 中的所有記錄", table_name);
                    } else {
                        // 解析 WHERE 子句
                        let condition = parts[1].trim();
                        let cond_parts: Vec<&str> = condition.split('=').collect();
                        if cond_parts.len() == 2 {
                            let col_name = cond_parts[0].trim();
                            let value = cond_parts[1].trim().trim_matches('\'').trim_matches('"');

                            // 找到列的索引
                            if let Some(col_idx) = table.columns.iter().position(|c| c.name == col_name) {
                                // 刪除匹配的記錄
                                let initial_len = table.rows.len();
                                table.rows.retain(|row| {
                                    row.values[col_idx].as_ref().map_or(true, |v| v != value)
                                });
                                let deleted_count = initial_len - table.rows.len();
                                self.output_text = format!("已刪除 {} 條記錄", deleted_count);
                            } else {
                                self.error_message = format!("列 '{}' 不存在", col_name);
                                return;
                            }
                        } else {
                            self.error_message = "無效的 WHERE 子句格式".to_string();
                            return;
                        }
                    }
                } else {
                    self.error_message = format!("表 '{}' 不存在", table_name);
                    return;
                }
            } else if query_upper.starts_with("UPDATE") {
                // 解析 UPDATE 語句
                let parts: Vec<&str> = query[6..].trim().split("SET").collect();
                if parts.len() != 2 {
                    self.error_message = "無效的 UPDATE 語句格式".to_string();
                    return;
                }

                let table_name = parts[0].trim();
                let remaining: Vec<&str> = parts[1].trim().split("WHERE").collect();
                
                if remaining.len() != 2 {
                    self.error_message = "UPDATE 語句必須包含 WHERE 子句".to_string();
                    return;
                }

                let set_clause = remaining[0].trim();
                let where_clause = remaining[1].trim();

                // 解析 SET 子句
                let set_parts: Vec<&str> = set_clause.split('=').collect();
                if set_parts.len() != 2 {
                    self.error_message = "無效的 SET 子句格式".to_string();
                    return;
                }

                let update_col = set_parts[0].trim();
                let new_value = set_parts[1].trim().trim_matches('\'').trim_matches('"');

                // 解析 WHERE 子句
                let where_parts: Vec<&str> = where_clause.split('=').collect();
                if where_parts.len() != 2 {
                    self.error_message = "無效的 WHERE 子句格式".to_string();
                    return;
                }

                let where_col = where_parts[0].trim();
                let where_value = where_parts[1].trim().trim_matches('\'').trim_matches('"');

                if let Some(table) = db.tables.iter_mut().find(|t| t.name == table_name) {
                    // 找到相關列的索引
                    let update_col_idx = table.columns.iter().position(|c| c.name == update_col);
                    let where_col_idx = table.columns.iter().position(|c| c.name == where_col);

                    match (update_col_idx, where_col_idx) {
                        (Some(update_idx), Some(where_idx)) => {
                            let mut update_count = 0;
                            for row in &mut table.rows {
                                if row.values[where_idx].as_ref().map_or(false, |v| v == where_value) {
                                    row.values[update_idx] = Some(new_value.to_string());
                                    update_count += 1;
                                }
                            }
                            self.output_text = format!("已更新 {} 條記錄", update_count);
                        }
                        _ => {
                            self.error_message = "指定的列不存在".to_string();
                            return;
                        }
                    }
                } else {
                    self.error_message = format!("表 '{}' 不存在", table_name);
                    return;
                }
            } else {
                self.error_message = "不支持的 SQL 命令".to_string();
                return;
            }
            self.save_database(); // 保存更改
        }
    }
}

pub fn run_gui() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        renderer: eframe::Renderer::default(),
        ..Default::default()
    };

    // 创建应用实例时设置字体
    eframe::run_native(
        "数据库管理系统",
        native_options,
        Box::new(|cc| {
            // 配置字体
            let mut fonts = egui::FontDefinitions::default();
            // 修改字体文件路径
            fonts.font_data.insert(
                "microsoft_yahei".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/msyh.ttf")),
            );
            
            fonts.families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "microsoft_yahei".to_owned());
            
            cc.egui_ctx.set_fonts(fonts);
            
            let mut gui = DatabaseGui::default();
            gui.load_database(); // 启动时加载数据库
            Ok(Box::new(gui))
        })
    )
}