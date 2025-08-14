#include <stdio.h>
#include "sqlite3.h"

int callback(void *, int, char **, char **);

int main(int argc, char **argv)
{
    // 1. 打开数据库
    sqlite3 *db = NULL;
    char *err_msg = NULL;

    int rc = sqlite3_open("/test.db", &db);
    if (rc != SQLITE_OK)
    {
        fprintf(stderr, "Cannot open database: %s\n", sqlite3_errmsg(db));
        sqlite3_close(db);
        return 1;
    }
    // 2. 写入数据
    const char *sql = "DROP TABLE IF EXISTS Cars;"
                      "CREATE TABLE Cars(Id INT, Name TEXT, Price INT);"
                      "INSERT INTO Cars VALUES(1, 'Audi', 52642);"
                      "INSERT INTO Cars VALUES(2, 'Skoda', 9000);";
    rc = sqlite3_exec(db, sql, NULL, NULL, &err_msg);
    if (rc != SQLITE_OK)
    {
        fprintf(stderr, "SQL error: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }

    sql = "SELECT * FROM Cars";
    rc = sqlite3_exec(db, sql, callback, NULL, &err_msg);
    if (rc != SQLITE_OK)
    {
        fprintf(stderr, "Failed to select data\n", err_msg);
        fprintf(stderr, "SQL error: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }


    sqlite3_close(db);

    return 0;
}

int callback(void *NotUsed, int argc, char **argv, char **azColName)
{
    NotUsed = NULL;
    for (int i = 0; i < argc; ++i)
    {
        printf("%s = %s\n", azColName[i], (argv[i]? argv[i]:"NULL"));
    }

    printf("\n");
    return 0;
}