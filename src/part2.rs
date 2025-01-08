use eframe::egui;
use crate::part1::{Database, Table, Row, DataType, Column};
use std::collections::HashMap;

pub struct DatabaseGui {
    database: Option<Database>,
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
    fn show_main_view(&mut self, ui: &mut egui::Ui) {
        if self.database.is_none() {
            if ui.button("创建数据库").clicked() {
                self.database = Some(Database::new("test_db"));
                self.output_text = "数据库创建成功！".to_string();
            }
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
                    });
                }
            }
        }
    }

    fn show_create_table_view(&mut self, ui: &mut egui::Ui) {
        if self.database.is_none() {
            ui.label("请先创建数���库！");
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
            self.execute_sql_query();
        }

        // 输出区域
        ui.group(|ui| {
            ui.label("查询结果：");
            ui.add(egui::TextEdit::multiline(&mut self.output_text)
                .interactive(false));
        });
    }

    fn execute_sql_query(&mut self) {
        if let Some(ref mut db) = self.database {
            let query = self.query_input.trim();
            let query_upper = query.to_uppercase();
    
            if query_upper.starts_with("SELECT") {
                // 解析表名
                if let Some(table_name) = query_upper
                    .split("FROM")
                    .nth(1)
                    .map(|s| s.trim().split_whitespace().next())
                    .flatten()
                {
                    // 查找表
                    if let Some(table) = db.tables.iter().find(|t| t.name.to_uppercase() == table_name) {
                        let mut output = String::new();
                        
                        // 添加表頭
                        for (i, col) in table.columns.iter().enumerate() {
                            if i > 0 {
                                output.push_str("\t");
                            }
                            output.push_str(&col.name);
                        }
                        output.push('\n');
                        
                        // 添加分隔線
                        for (i, col) in table.columns.iter().enumerate() {
                            if i > 0 {
                                output.push_str("\t");
                            }
                            output.push_str(&"-".repeat(col.name.len()));
                        }
                        output.push('\n');
                        
                        // 添加數據行
                        for row in &table.rows {
                            for (i, value) in row.values.iter().enumerate() {
                                if i > 0 {
                                    output.push_str("\t");
                                }
                                match value {
                                    Some(v) => output.push_str(v),
                                    None => output.push_str("NULL"),
                                }
                            }
                            output.push('\n');
                        }
                        
                        // 添加總行數
                        output.push('\n');
                        output.push_str(&format!("Total rows: {}", table.rows.len()));
                        
                        self.output_text = output;
                    } else {
                        self.error_message = format!("Table '{}' does not exist!", table_name);
                    }
                } else {
                    self.error_message = "Invalid SELECT statement syntax!".to_string();
                }
            } else if query_upper.starts_with("INSERT INTO") {
                let parts: Vec<&str> = query.split("VALUES").collect();
                if parts.len() == 2 {
                    let table_name = parts[0]
                        .strip_prefix("INSERT INTO")
                        .map(|s| s.trim())
                        .unwrap_or("");

                    // 查找表
                    if let Some(table_index) = db.tables.iter().position(|t| t.name == table_name) {
                        let values_str = parts[1].trim();
                        if values_str.starts_with('(') && values_str.ends_with(')') {
                            let values: Vec<String> = values_str[1..values_str.len()-1]
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .collect();

                            if values.len() == db.tables[table_index].columns.len() {
                                let mut processed_values = Vec::new();
                                
                                // 處理每個值
                                for (value, column) in values.iter().zip(db.tables[table_index].columns.iter()) {
                                    let processed_value = match column.data_type {
                                        DataType::Int => {
                                            if let Ok(_) = value.parse::<i32>() {
                                                Some(value.clone())
                                            } else {
                                                self.error_message = format!("Invalid integer value: {}", value);
                                                return;
                                            }
                                        },
                                        DataType::Bool => {
                                            if value.to_lowercase() == "true" || value.to_lowercase() == "false" {
                                                Some(value.clone())
                                            } else {
                                                self.error_message = format!("Invalid boolean value: {}", value);
                                                return;
                                            }
                                        },
                                        DataType::String(_) | DataType::Char(_) => {
                                            Some(value.trim_matches(|c| c == '\'' || c == '"').to_string())
                                        },
                                    };
                                    processed_values.push(processed_value);
                                }

                                // 直接修改表中的數據
                                let new_row = Row {
                                    values: processed_values,
                                };
                                db.tables[table_index].rows.push(new_row);
                                self.output_text = format!("Successfully inserted 1 row into table '{}'", table_name);
                            } else {
                                self.error_message = format!(
                                    "Column count doesn't match. Expected {}, got {}",
                                    db.tables[table_index].columns.len(),
                                    values.len()
                                );
                            }
                        } else {
                            self.error_message = "Invalid VALUES syntax!".to_string();
                        }
                    } else {
                        self.error_message = format!("Table '{}' does not exist!", table_name);
                    }
                } else {
                    self.error_message = "Invalid INSERT statement syntax!".to_string();
                }
            } else if query_upper.starts_with("UPDATE") {
                let parts: Vec<&str> = query.split("SET").collect();
                if parts.len() == 2 {
                    let table_name = parts[0]
                        .strip_prefix("UPDATE")
                        .map(|s| s.trim())
                        .unwrap_or("");

                    let set_where: Vec<&str> = parts[1].split("WHERE").collect();
                    if set_where.len() >= 1 {
                        let set_clause = set_where[0].trim();
                        let where_clause = set_where.get(1).map(|s| s.trim());

                        // 直接獲取表的可變引用
                        if let Some(table) = db.tables.iter_mut().find(|t| t.name == table_name) {
                            let set_pairs: Vec<(&str, &str)> = set_clause
                                .split(',')
                                .filter_map(|pair| {
                                    let kv: Vec<&str> = pair.split('=').map(|s| s.trim()).collect();
                                    if kv.len() == 2 {
                                        Some((kv[0], kv[1]))
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            let mut updated_count = 0;
                            
                            // 創建列名到索引的映射
                            let column_indices: HashMap<&str, usize> = table.columns
                                .iter()
                                .enumerate()
                                .map(|(i, col)| (col.name.as_str(), i))
                                .collect();

                            // 更新符合條件的行
                            for row in &mut table.rows {
                                let should_update = match where_clause {
                                    None => true,
                                    Some(where_cond) => {
                                        let cond_parts: Vec<&str> = where_cond.split('=').map(|s| s.trim()).collect();
                                        if cond_parts.len() == 2 {
                                            if let Some(&col_index) = column_indices.get(cond_parts[0]) {
                                                if let Some(Some(value)) = row.values.get(col_index) {
                                                    let cond_value = cond_parts[1].trim_matches(|c| c == '\'' || c == '"');
                                                    value == cond_value
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    }
                                };

                                if should_update {
                                    for (col_name, new_value) in &set_pairs {
                                        if let Some(&col_index) = column_indices.get(*col_name) {
                                            let cleaned_value = new_value.trim_matches(|c| c == '\'' || c == '"').to_string();
                                            row.values[col_index] = Some(cleaned_value);
                                        }
                                    }
                                    updated_count += 1;
                                }
                            }
                            self.output_text = format!("Successfully updated {} rows in table '{}'", updated_count, table_name);
                        } else {
                            self.error_message = format!("Table '{}' does not exist!", table_name);
                        }
                    }
                }
            } else if query_upper.starts_with("DELETE FROM") {
                let parts: Vec<&str> = query.split("WHERE").collect();
                let table_name = parts[0]
                    .strip_prefix("DELETE FROM")
                    .map(|s| s.trim())
                    .unwrap_or("");

                // 直接獲取表的可變引用
                if let Some(table) = db.tables.iter_mut().find(|t| t.name == table_name) {
                    let where_clause = parts.get(1).map(|s| s.trim());
                    
                    let initial_count = table.rows.len();
                    
                    // 根據條件刪除行
                    if let Some(where_cond) = where_clause {
                        let cond_parts: Vec<&str> = where_cond.split('=').map(|s| s.trim()).collect();
                        if cond_parts.len() == 2 {
                            let col_name = cond_parts[0].trim();
                            if let Some(col_index) = table.columns.iter().position(|c| c.name == col_name) {
                                let cond_value = cond_parts[1].trim_matches(|c| c == '\'' || c == '"');
                                table.rows.retain(|row| {
                                    if let Some(Some(value)) = row.values.get(col_index) {
                                        value != cond_value
                                    } else {
                                        true
                                    }
                                });
                            }
                        }
                    } else {
                        // 沒有 WHERE 子句時刪除所有行
                        table.rows.clear();
                    }
                    
                    let deleted_count = initial_count - table.rows.len();
                    self.output_text = format!("Successfully deleted {} rows from table '{}'", deleted_count, table_name);
                } else {
                    self.error_message = format!("Table '{}' does not exist!", table_name);
                }
            } else {
                self.error_message = "Unsupported SQL command!".to_string();
            }
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
            
            Ok(Box::new(DatabaseGui::default()))
        })
    )
}