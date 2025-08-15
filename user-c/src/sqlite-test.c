#include <stdio.h>
#include <sqlite3.h>

// Callback function for queries
static int callback(void *NotUsed, int argc, char **argv, char **colName) {
    for (int i = 0; i < argc; i++) {
        printf("%s = %s\n", colName[i], argv[i] ? argv[i] : "NULL");
    }
    printf("\n");
    return 0;
}

int main() {
    sqlite3 *db;
    char *err_msg = NULL;
    int rc;

    // 1. Open or create a database
    rc = sqlite3_open("test.db", &db);
    if (rc != SQLITE_OK) {
        fprintf(stderr, "Cannot open database: %s\n", sqlite3_errmsg(db));
        sqlite3_close(db);
        return 1;
    }

    // 2. Create a table
    const char *sql = "DROP TABLE IF EXISTS Users;"
                      "CREATE TABLE Users(Id INT PRIMARY KEY, Name TEXT);"
                      "INSERT INTO Users VALUES(1, 'Alice');"
                      "INSERT INTO Users VALUES(2, 'Bob');";
    
    rc = sqlite3_exec(db, sql, 0, 0, &err_msg);
    if (rc != SQLITE_OK) {
        fprintf(stderr, "SQL error: %s\n", err_msg);
        sqlite3_free(err_msg);
        sqlite3_close(db);
        return 1;
    }

    // 3. Query the data
    printf("Database contents:\n");
    sql = "SELECT * FROM Users";
    rc = sqlite3_exec(db, sql, callback, 0, &err_msg);
    if (rc != SQLITE_OK) {
        fprintf(stderr, "Query failed: %s\n", err_msg);
        sqlite3_free(err_msg);
    }

    // 4. Close database
    sqlite3_close(db);
    return 0;
}

