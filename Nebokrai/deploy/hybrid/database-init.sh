#!/bin/bash
set -euo pipefail

sqlcmd=/opt/mssql-tools18/bin/sqlcmd

until "$sqlcmd" -S mssql -U sa -P "$MSSQL_SA_PASSWORD" -C -Q "SELECT 1" >/dev/null 2>&1; do
    echo "Database: ожидание MSSQL"
    sleep 2
done

restore_database() {
    database="$1"
    backup="$2"
    data_name="$3"
    log_name="$4"

    "$sqlcmd" -S mssql -U sa -P "$MSSQL_SA_PASSWORD" -C -b -Q "
IF DB_ID(N'$database') IS NULL
BEGIN
    RESTORE DATABASE [$database]
      FROM DISK = N'/backup/$backup'
      WITH MOVE N'$data_name' TO N'/var/opt/mssql/data/$database.mdf',
           MOVE N'$log_name' TO N'/var/opt/mssql/data/${database}_log.ldf',
           RECOVERY, REPLACE;
END"
}

restore_database Account Account.bak AccountDB_dat AccountDB_log
restore_database BillingDB BillingDB.bak BillingDB_Data BillingDB_Log
restore_database GameDB GameDB05.bak FY_GameDB05_dat FY_GameDB05_log
restore_database LogDB LogDB.bak LogDB_dat LogDB_log
restore_database LoginDB LoginDB.bak LoginDB_dat LoginDB_log

"$sqlcmd" -S mssql -U sa -P "$MSSQL_SA_PASSWORD" -C -b \
    -v MIRACLE_PASSWORD="$MIRACLE_DB_PASSWORD" -Q "
DECLARE @password nvarchar(128) = N'\$(MIRACLE_PASSWORD)';
DECLARE @statement nvarchar(max);
IF SUSER_ID(N'Miracle') IS NULL
BEGIN
    SET @statement = N'CREATE LOGIN [Miracle] WITH PASSWORD = ' + QUOTENAME(@password, N'''') + N', CHECK_POLICY = OFF';
    EXEC sys.sp_executesql @statement;
END
ELSE
BEGIN
    SET @statement = N'ALTER LOGIN [Miracle] WITH PASSWORD = ' + QUOTENAME(@password, N'''') + N', CHECK_POLICY = OFF';
    EXEC sys.sp_executesql @statement;
END

DECLARE @database sysname;
DECLARE databases CURSOR LOCAL FAST_FORWARD FOR
    SELECT name FROM (VALUES (N'Account'), (N'BillingDB'), (N'GameDB'), (N'LogDB'), (N'LoginDB')) AS names(name);
OPEN databases;
FETCH NEXT FROM databases INTO @database;
WHILE @@FETCH_STATUS = 0
BEGIN
    SET @statement = N'USE ' + QUOTENAME(@database) + N';
        IF USER_ID(N''Miracle'') IS NULL CREATE USER [Miracle] FOR LOGIN [Miracle];
        ELSE ALTER USER [Miracle] WITH LOGIN = [Miracle];
        IF IS_ROLEMEMBER(N''db_owner'', N''Miracle'') <> 1 ALTER ROLE [db_owner] ADD MEMBER [Miracle];';
    EXEC sys.sp_executesql @statement;
    FETCH NEXT FROM databases INTO @database;
END
CLOSE databases;
DEALLOCATE databases;"

echo "Database: пять баз восстановлены, login Miracle связан"
