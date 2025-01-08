use serde::{Serialize, Deserialize};

// 定义数据类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Int,
    Char(u32), // 字符长度
    Bool,
    String(u32), // 字符串长度
}

// 定义列结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,       // 将字段设为公有
    pub data_type: DataType,    // 数据类型
    pub is_primary_key: bool,   // 是否为主键
}

// 定义行结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Row {
    pub values: Vec<Option<String>>, // 存储每一列的值，使用 Option 处理可能的空值
}

// 定义表结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub name: String,            // 将字段设为公有
    pub columns: Vec<Column>,    // 将字段设为公有
    pub rows: Vec<Row>,          // 将字段设为公有
}

// 定义数据库结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    pub tables: Vec<Table>,      // 将此字段设为公有
}

// 创建数据库
impl Database {
    pub fn new(name: &str) -> Self {
        Database {
            tables: Vec::new(),
        }
    }

    // 创建表
    pub fn create_table(&mut self, table: Table) {
        self.tables.push(table);
    }

    // 插入行
    pub fn insert_row(&mut self, table_name: &str, row: Row) {
        if let Some(table) = self.tables.iter_mut().find(|t| t.name == table_name) {
            table.rows.push(row);
        }
    }

    // 读取行
    pub fn read_rows(&self, table_name: &str) -> Option<&Vec<Row>> {
        self.tables.iter().find(|t| t.name == table_name).map(|t| &t.rows)
    }

    // 更新行
    pub fn update_row(&mut self, table_name: &str, row_index: usize, new_row: Row) {
        if let Some(table) = self.tables.iter_mut().find(|t| t.name == table_name) {
            if row_index < table.rows.len() {
                table.rows[row_index] = new_row;
            }
        }
    }

    // 删除行
    pub fn delete_row(&mut self, table_name: &str, row_index: usize) {
        if let Some(table) = self.tables.iter_mut().find(|t| t.name == table_name) {
            if row_index < table.rows.len() {
                table.rows.remove(row_index);
            }
        }
    }
}
