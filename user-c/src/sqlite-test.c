#include <stdio.h>
#include <stdlib.h>
#include <sqlite3.h>

// 回调函数，用于查询结果的输出
static int callback(void *NotUsed, int argc, char **argv, char **azColName) {
    int i;
    for(i = 0; i < argc; i++) {
        printf("%s = %s\n", azColName[i], argv[i] ? argv[i] : "NULL");
    }
    printf("\n");
    return 0;
}

int main(int argc, char* argv[]) {
    sqlite3 *db;
    char *err_msg = NULL;
    int rc;
    const char *sql;

    // 打开或创建数据库
    rc = sqlite3_open("test.db", &db);
    if(rc != SQLITE_OK) {
        fprintf(stderr, "无法打开数据库: %s\n", sqlite3_errmsg(db));
        sqlite3_close(db);
        return 1;
    }

    // 创建表
    sql = "DROP TABLE IF EXISTS COMPANY;"
          "CREATE TABLE COMPANY("
          "ID INT PRIMARY KEY     NOT NULL,"
          "NAME           TEXT    NOT NULL,"
          "AGE            INT     NOT NULL,"
          "ADDRESS        CHAR(50),"
          "SALARY         REAL );";
    
    rc = sqlite3_exec(db, sql, 0, 0, &err_msg);
    if(rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }

    // 插入数据
    sql = "INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY) "
          "VALUES (1, 'Paul', 32, 'California', 20000.00 ); "
          "INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY) "
          "VALUES (2, 'Allen', 25, 'Texas', 15000.00 ); "
          "INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)"
          "VALUES (3, 'Teddy', 23, 'Norway', 20000.00 );"
          "INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)"
          "VALUES (4, 'Mark', 25, 'Rich-Mond ', 65000.00 );";
    
    rc = sqlite3_exec(db, sql, 0, 0, &err_msg);
    if(rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }

    // 查询数据
    printf("查询所有数据:\n");
    sql = "SELECT * FROM COMPANY;";
    rc = sqlite3_exec(db, sql, callback, 0, &err_msg);
    if(rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }

    // 更新数据
    sql = "UPDATE COMPANY set SALARY = 25000.00 where ID=1;";
    rc = sqlite3_exec(db, sql, 0, 0, &err_msg);
    if(rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }

    printf("更新后的数据:\n");
    sql = "SELECT * FROM COMPANY;";
    rc = sqlite3_exec(db, sql, callback, 0, &err_msg);
    if(rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }

    // 删除数据
    sql = "DELETE from COMPANY where ID=2;";
    rc = sqlite3_exec(db, sql, 0, 0, &err_msg);
    if(rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }

    printf("删除后的数据:\n");
    sql = "SELECT * FROM COMPANY;";
    rc = sqlite3_exec(db, sql, callback, 0, &err_msg);
    if(rc != SQLITE_OK) {
        fprintf(stderr, "SQL错误: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }

    // 关闭数据库
    sqlite3_close(db);
    printf("数据库操作完成\n");
    return 0;
}
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

