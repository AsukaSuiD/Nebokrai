#pragma once

#include "rscdkey.h"
#include "../odbc.h"

#include <string>

/*
 * Технический Linux-адаптер CRsCDKey::Database. Он заменяет только ADO/COM:
 * SQL и интерпретация строк принадлежат rscdkey.cpp. Каждая Query/Execute
 * открывает отдельное MSSQL ODBC-соединение, как исходный владелец.
 */
class MssqlOdbcRsCdKeyDatabase final : public CRsCDKey::Database
{
public:
    MssqlOdbcRsCdKeyDatabase(std::string host,
                             std::string database,
                             std::string user,
                             std::string password,
                             Nebokrai::Database::MssqlConnectionOptions options = {});

    [[nodiscard]] std::vector<Row> Query(const std::string& sql) override;
    [[nodiscard]] bool Execute(const std::string& sql) override;
    void PutStringToFile(const std::string& category,
                         const std::string& message) override;
    void PrintErr(const std::string& operation,
                  const std::exception& error) override;

private:
    [[nodiscard]] std::string ConnectionString() const;

    std::string m_Host;
    std::string m_Database;
    std::string m_User;
    std::string m_Password;
    Nebokrai::Database::MssqlConnectionOptions m_Options;
};
