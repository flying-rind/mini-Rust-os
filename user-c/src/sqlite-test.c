#include <stdio.h>
#include <sqlite3.h>

 // 4. 查询数据回调函数
int callback(void *data, int argc, char **argv, char **col_name) {
    for (int i = 0; i < argc; i++) {
        printf("%s = %s\n", col_name[i], argv[i] ? argv[i] : "NULL");
    }
    printf("\n");
    return 0;
}

int main() {
    sqlite3 *db;
    char *err_msg = NULL;
    
    // 1. 打开内存数据库（无需文件IO权限）
    int rc = sqlite3_open(":memory:", &db);
    if (rc != SQLITE_OK) {
        fprintf(stderr, "无法打开数据库: %s\n", sqlite3_errmsg(db));
        return 1;
    }
    
    // 2. 创建测试表
    const char *create_table_sql = 
        "CREATE TABLE IF NOT EXISTS Test("
        "id INTEGER PRIMARY KEY,"
        "name TEXT NOT NULL);";
    
    rc = sqlite3_exec(db, create_table_sql, 0, 0, &err_msg);
    if (rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }
    
    // 3. 插入数据
    const char *insert_sql = 
        "INSERT INTO Test(name) VALUES"
        "('Alice'), ('Bob'), ('Charlie');";
    
    rc = sqlite3_exec(db, insert_sql, 0, 0, &err_msg);
    if (rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }
    
   
    
    // 5. 执行查询
    printf("--- 测试数据 ---\n");
    const char *select_sql = "SELECT * FROM Test;";
    rc = sqlite3_exec(db, select_sql, callback, 0, &err_msg);
    if (rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
    }
    
    // 6. 打印SQLite版本
    printf("SQLite版本: %s\n", sqlite3_libversion());
    
    // 关闭数据库
    sqlite3_close(db);
    return 0;
}

