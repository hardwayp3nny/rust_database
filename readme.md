# 創建 README.md 文件
echo '# Rust Database

一個使用 Rust 實現的簡單數據庫系統，具有圖形用戶界面。

## 功能特點

- 基於 egui 的圖形界面
- 支持基本的 SQL 操作（SELECT、INSERT、DELETE、UPDATE）
- 數據庫完整性驗證
- JSON 格式數據存儲
- 表格創建和管理
- 數據插入和查詢界面

## 支持的 SQL 命令
sql
-- SELECT 查詢
SELECT FROM table_name
SELECT FROM table1 AND table2
-- INSERT 插入
INSERT INTO table_name VALUES (value1, value2, ...)
-- DELETE 刪除
DELETE FROM table_name WHERE column = value
DELETE FROM table_name
-- UPDATE 更新
UPDATE table_name SET column = value WHERE column = value
## 運行方式
bash
cargo run

## 依賴項

- eframe
- egui
- serde
- serde_json
- sha2

## 許可證

MIT' > README.md

