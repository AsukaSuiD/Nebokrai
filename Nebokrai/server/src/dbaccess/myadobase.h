#pragma once

#include <cstddef>
#include <memory>
#include <string>
#include <string_view>

namespace Nebokrai::Database
{
class OdbcConnection;
}

/*
 * Исходный владелец: dbaccess/myadobase.cpp / myadobase.h.
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 *
 * Материализован только connection-slice, уже необходимый AccLogThread:
 * Uninitalize 0x004641E0, GetTimeString 0x00464210,
 * CreateCn 0x00464450, OpenCn 0x00464650, CloseCn 0x00464720,
 * ExecuteCn 0x004647E0, ReleaseCn 0x00464B40, Initialize 0x00464CB0.
 * Владелец Recordset (CreateRs/OpenRs/CloseRs/ReleaseRs) намеренно не объявляется
 * до восстановления его реальных вызывающих сторон.
 *
 * Старый транспорт ADO/COM заменён unixODBC. Initialize всё ещё сохраняет все
 * семь исходных string и строит ТОЧНУЮ ADO-строку владельца; таймаут подключения и
 * integrated security исходник только сохранял и в connection string не
 * добавлял. Для фактического Linux OpenCn из тех же server/database/user/password
 * строится техническая ODBC-строка. Это не новая DB/state-machine семантика.
 */
class CMyAdoBase
{
public:
    struct Connection
    {
        Connection();
        ~Connection();

        Connection(const Connection&) = delete;
        Connection& operator=(const Connection&) = delete;

        std::unique_ptr<Nebokrai::Database::OdbcConnection> implementation;
        bool created{};
        bool open{};
        std::string lastError;
    };

    CMyAdoBase() = default;
    virtual ~CMyAdoBase() = default;

    [[nodiscard]] static bool Initialize(std::string provider,
                                         std::string dataSource,
                                         std::string initialCatalog,
                                         std::string userId,
                                         std::string password,
                                         std::string connectTimeout,
                                         std::string integratedSecurity);
    [[nodiscard]] static bool Uninitalize() noexcept;

    [[nodiscard]] static bool CreateCn(Connection& connection);
    [[nodiscard]] static bool OpenCn(Connection& connection);
    [[nodiscard]] static bool ExecuteCn(const char* sql, Connection& connection);
    [[nodiscard]] static bool ExecuteAccountEnterLog(std::string_view account,
                                                     std::string_view enteredAt,
                                                     std::string_view ip,
                                                     Connection& connection);
    [[nodiscard]] static bool CloseCn(Connection& connection);
    static void ReleaseCn(Connection& connection) noexcept;

// В прямом API был только char*. Все исходные вызывающие стороны передавали
// стековые массивы фиксированного размера; перегрузка с ёмкостью — безопасная
// точка входа Linux, которой пользуется восстановленный код.
    [[nodiscard]] static char* GetTimeString(char* buffer, std::size_t capacity);

private:
    static std::string m_strConnectionString;
    static std::string m_strProvider;
    static std::string m_strDataSource;
    static std::string m_strInitialCatalog;
    static std::string m_strUserID;
    static std::string m_strPassword;
    static std::string m_strConnectTimeout;
    static std::string m_strIntegratedSecurity;
};
