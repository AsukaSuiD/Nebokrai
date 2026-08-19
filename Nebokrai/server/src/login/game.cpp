#include "game.h"

#include "loginqueue.h"
#include "../nets/netlogin/message.h"
#include "../nets/netlogin/mynetserver_client.h"
#include "../nets/netlogin/mynetserver_world.h"

#include <iconv.h>
#include <sql.h>
#include <sqlext.h>

#include <algorithm>
#include <bit>
#include <cerrno>
#include <chrono>
#include <cstring>
#include <ctime>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace Login
{
namespace
{
constexpr std::int32_t kLoginResponseMessageType = 0x000AF501;
constexpr std::int32_t kPlayerBaseMessageType = 0x0004FB01;
constexpr std::int32_t kKickWorldAccountMessageType = 0x0004FB07;
constexpr std::size_t kLegacyAccountLogBufferSize = 0x200U;
constexpr std::size_t kLegacyExecuteConnectionBufferSize = 0x400U;

std::span<const std::uint8_t> CStringBytes(const char* value)
{
    if (value == nullptr) {
        return {};
    }
    return {reinterpret_cast<const std::uint8_t*>(value), std::strlen(value)};
}

std::vector<std::uint8_t> OwnedCStringBytes(const char* value)
{
    const auto bytes = CStringBytes(value);
    return {bytes.begin(), bytes.end()};
}

std::string LegacyIpv4Text(std::uint32_t address)
{
    return std::to_string(address & 0xFFU) + '.' +
           std::to_string((address >> 8U) & 0xFFU) + '.' +
           std::to_string((address >> 16U) & 0xFFU) + '.' +
           std::to_string((address >> 24U) & 0xFFU);
}

std::optional<std::string> LegacyLocalTimeText()
{
    const std::time_t now = std::chrono::system_clock::to_time_t(
        std::chrono::system_clock::now());
    std::tm local{};
#if defined(_WIN32)
    if (::localtime_s(&local, &now) != 0) {
        return std::nullopt;
    }
#else
    if (::localtime_r(&now, &local) == nullptr) {
        return std::nullopt;
    }
#endif
    // VERIFIED CMyAdoBase::GetTimeString 0x00464210:
    // "%d-%d-%d %d:%d:%d", без zero-padding.
    return std::to_string(local.tm_year + 1900) + '-' +
           std::to_string(local.tm_mon + 1) + '-' +
           std::to_string(local.tm_mday) + ' ' +
           std::to_string(local.tm_hour) + ':' +
           std::to_string(local.tm_min) + ':' +
           std::to_string(local.tm_sec);
}

struct ConvertedText
{
    std::optional<std::string> value;
    std::string error;
};

ConvertedText Windows1251ToUtf8(std::string_view source)
{
    if (source.empty()) {
        return ConvertedText{.value = std::string{}};
    }

    iconv_t converter = iconv_open("UTF-8", "WINDOWS-1251");
    if (converter == reinterpret_cast<iconv_t>(-1)) {
        return ConvertedText{.error = "iconv WINDOWS-1251 -> UTF-8 unavailable"};
    }
    struct Closer
    {
        iconv_t converter;
        ~Closer() { iconv_close(converter); }
    } closer{converter};

    std::string output(source.size() * 4U + 4U, '\0');
    char* input = const_cast<char*>(source.data());
    std::size_t inputLeft = source.size();
    char* destination = output.data();
    std::size_t destinationLeft = output.size();
    errno = 0;
    if (iconv(converter,
              &input,
              &inputLeft,
              &destination,
              &destinationLeft) == static_cast<std::size_t>(-1)) {
        return ConvertedText{
            .error = "iconv WINDOWS-1251 -> UTF-8 failed, errno=" +
                     std::to_string(errno)};
    }
    output.resize(output.size() - destinationLeft);
    return ConvertedText{.value = std::move(output)};
}

bool OdbcSucceeded(SQLRETURN result) noexcept
{
    return result == SQL_SUCCESS || result == SQL_SUCCESS_WITH_INFO;
}

std::string OdbcDiagnostic(SQLSMALLINT handleType,
                           SQLHANDLE handle,
                           std::string_view operation)
{
    SQLCHAR state[6]{};
    SQLINTEGER native = 0;
    SQLCHAR message[1024]{};
    SQLSMALLINT length = 0;
    const SQLRETURN result = SQLGetDiagRec(handleType,
                                            handle,
                                            1,
                                            state,
                                            &native,
                                            message,
                                            static_cast<SQLSMALLINT>(sizeof(message)),
                                            &length);
    std::string detail(operation);
    if (OdbcSucceeded(result)) {
        detail += ": SQLSTATE=";
        detail += reinterpret_cast<const char*>(state);
        detail += ", native=" + std::to_string(native);
        if (length > 0) {
            detail += ", ";
            const std::size_t copied = std::min<std::size_t>(
                static_cast<std::size_t>(length), sizeof(message) - 1U);
            detail.append(reinterpret_cast<const char*>(message), copied);
        }
    } else {
        detail += ": ODBC diagnostic unavailable";
    }
    return detail;
}
}

std::int32_t CGame::GetWorldIDByName(const char* worldName) const
{
    if (worldName == nullptr) {
        return -1;
    }
    for (const auto& [worldId, world] : m_listWorldInfo) {
        if (std::strcmp(worldName, world.strName.c_str()) == 0) {
            return world.lStateLvl == 0 ? -1 : worldId;
        }
    }
    return -1;
}

bool CGame::WorldServerIsOpenState(std::int32_t worldId) const
{
    const auto found = m_WorldInfoSetup.find(worldId);
    return found != m_WorldInfoSetup.end() && found->second.lStateLvl != 0;
}

void CGame::AddWorldInfoToMsg(LoginNet::CMessage& message, const char* account) const
{
    // VERIFIED_ASSEMBLY 0x004074E0: map size идёт short, затем для каждого
    // live world — long state и C-string name в map-key order.
    const auto lowCount = static_cast<std::uint16_t>(m_listWorldInfo.size());
    message.Base().Add(std::bit_cast<std::int16_t>(lowCount));

    for (const auto& [worldId, world] : m_listWorldInfo) {
        const auto setup = m_WorldInfoSetup.find(worldId);
        bool visible =
            setup != m_WorldInfoSetup.end() && setup->second.lStateLvl != 0;
        if (!visible && m_pLoginQueue != nullptr) {
            visible = m_pLoginQueue->IsInNoQueueList(CStringBytes(account));
        }
        message.Base().Add(visible ? world.lStateLvl : 0);
        message.Base().Add(world.strName.c_str());
    }
}

std::int32_t CGame::FindCdkey(const char* account) const
{
    if (account == nullptr) {
        return -1;
    }
    for (const auto& [worldId, accounts] : s_listCdkey) {
        if (std::find(accounts.begin(), accounts.end(), account) != accounts.end()) {
            return worldId;
        }
    }
    return -1;
}

bool CGame::L2W_PlayerBase_Send(const char* worldName, const char* account) const
{
    if (worldName == nullptr || account == nullptr || account[0] == '\0') {
        return false;
    }

    const std::int32_t worldId = GetWorldIDByName(worldName);
    if (worldId == -1) {
        return false;
    }

    if (!WorldServerIsOpenState(worldId)) {
        const bool noQueue =
            m_pLoginQueue != nullptr &&
            m_pLoginQueue->IsInNoQueueList(CStringBytes(account));
        if (!noQueue) {
            return false;
        }
    }

    LoginNet::CMessage message(kPlayerBaseMessageType);
    message.Base().Add(account);
    if (s_pNetServer_World != nullptr) {
        static_cast<void>(
            message.SendToWorldMap(s_pNetServer_World->CommandHandle(), worldId));
    }
    // Direct EXE возвращает true после логических проверок и не анализирует send.
    return true;
}

const char* CGame::GetLoginCdkeyWorldServer(const char* account) const
{
    if (account == nullptr) {
        return nullptr;
    }
    const auto found = m_LoginCdkeyWorld.find(account);
    return found == m_LoginCdkeyWorld.end() ? nullptr : found->second.c_str();
}

void CGame::SetLoginCdkeyWorldServer(const char* account, const char* worldServer)
{
    if (account == nullptr || worldServer == nullptr) {
        return;
    }
    m_LoginCdkeyWorld[account] = worldServer;
}

bool CGame::KickOut(const char* account) const
{
    if (account == nullptr) {
        return false;
    }

    if (GetLoginCdkeyWorldServer(account) != nullptr) {
        if (s_pNetServer_Client != nullptr) {
            static_cast<void>(
                s_pNetServer_Client->QuitClientByMapName(CStringBytes(account)));
        }
        return true;
    }

    const std::int32_t worldId = FindCdkey(account);
    if (worldId == -1) {
        return false;
    }

    LoginNet::CMessage message(kKickWorldAccountMessageType);
    message.Base().Add(account);
    if (s_pNetServer_World != nullptr) {
        static_cast<void>(
            message.SendToWorldMap(s_pNetServer_World->CommandHandle(), worldId));
    }
    return true;
}

void CGame::AccountEnterLog(const char* account, std::uint32_t ip)
{
    if (account == nullptr) {
        return;
    }

    const auto time = LegacyLocalTimeText();
    if (!time) {
        RecordTechnicalError("AccountEnterLog: localtime conversion failed");
        return;
    }

    const std::string ipText = LegacyIpv4Text(ip);
    std::string sql = "INSERT INTO LogInfo(Account,AccountEnterTime,IP) VALUES('";
    sql += account;
    sql += "','";
    sql += *time;
    sql += "','";
    sql += ipText;
    sql += "')";

    // Direct owner uses char[512] + sprintf. Reached login accounts are <=31
    // bytes, therefore normal behavior is byte-identical. Oversize external
    // input is an explicit technical boundary instead of reproducing stack OOB.
    if (sql.size() >= kLegacyAccountLogBufferSize) {
        RecordTechnicalError("AccountEnterLog: SQL exceeds legacy char[512]");
        return;
    }
    _acc_logs.Push(std::move(sql));
}

std::int32_t CGame::PrepareEnter(const char* account,
                                 std::uint32_t ip,
                                 std::int32_t socketId,
                                 const char* worldServer,
                                 bool matrix)
{
    if (account == nullptr || ip == 0U || socketId == 0 || worldServer == nullptr) {
        return 1;
    }

    if (worldServer[0] == '\0' && KickOut(account)) {
        // VERIFIED_ASSEMBLY 0x0041198C..0x004119B4: unlike old Linux donor,
        // original sends 0x08 to the NEW socket and returns immediately.
        LoginNet::CMessage response(kLoginResponseMessageType);
        response.Base().Add(static_cast<char>(0x08));
        if (s_pNetServer_Client != nullptr) {
            static_cast<void>(response.SendToClientSocket(
                s_pNetServer_Client->CommandHandle(), socketId));
        }
        return 1;
    }

    if (s_pNetServer_Client != nullptr) {
        static_cast<void>(
            s_pNetServer_Client->SetClientMapName(socketId, CStringBytes(account)));
    } else {
        RecordTechnicalError("PrepareEnter: client server owner is missing");
    }

    AccountEnterLog(account, ip);
    SetLoginCdkeyWorldServer(account, worldServer);

    if (worldServer[0] != '\0' || !matrix) {
        return 0;
    }

    if (m_pLoginQueue == nullptr) {
        RecordTechnicalError("PrepareEnter: login queue owner is missing for matrix_register");
        return 1;
    }

    TagPwdChecked checked(socketId,
                          ip,
                          OwnedCStringBytes(account),
                          OwnedCStringBytes(worldServer),
                          true);
    if (auto error = m_pLoginQueue->MatrixRegister(
            checked,
            [this](const LoginNet::CMessage& message, std::int32_t targetSocket) {
                if (s_pNetServer_Client != nullptr) {
                    static_cast<void>(message.SendToClientSocket(
                        s_pNetServer_Client->CommandHandle(), targetSocket));
                }
            })) {
        RecordTechnicalError("PrepareEnter/matrix_register: " + error->detail);
    }
    return 1;
}

void CGame::EnterGame(const char* account,
                      std::uint32_t ip,
                      std::int32_t socketId,
                      const char* worldServer) const
{
    if (account == nullptr || ip == 0U || socketId == 0 || worldServer == nullptr) {
        return;
    }

    if (worldServer[0] != '\0') {
        static_cast<void>(L2W_PlayerBase_Send(worldServer, account));
        return;
    }

    LoginNet::CMessage response(kLoginResponseMessageType);
    response.Base().Add(static_cast<char>(0x02));
    response.Base().Add(account);
    AddWorldInfoToMsg(response, account);
    if (s_pNetServer_Client != nullptr) {
        static_cast<void>(response.SendToClientSocket(
            s_pNetServer_Client->CommandHandle(), socketId));
    }
}

bool CGame::ExecuteProce(std::string userId,
                         std::string userIp,
                         char* passwordHex,
                         std::int32_t unusedResult)
{
    // VERIFIED_ASSEMBLY 0x00407AB0..0x0040822C: этот аргумент не читается.
    static_cast<void>(unusedResult);
    if (passwordHex == nullptr) {
        RecordTechnicalError("ExecuteProce: UserPwd pointer is null");
        return false;
    }

    const ConvertedText host = Windows1251ToUtf8(m_Setup._db_ip);
    const ConvertedText database = Windows1251ToUtf8(m_Setup._db_billing_name);
    const ConvertedText dbUser = Windows1251ToUtf8(m_Setup._db_user);
    const ConvertedText dbPassword = Windows1251ToUtf8(m_Setup._db_psd);
    const ConvertedText convertedUser = Windows1251ToUtf8(userId);
    const ConvertedText convertedIp = Windows1251ToUtf8(userIp);
    const ConvertedText convertedPassword = Windows1251ToUtf8(passwordHex);

    const ConvertedText* conversions[] = {
        &host, &database, &dbUser, &dbPassword,
        &convertedUser, &convertedIp, &convertedPassword,
    };
    for (const ConvertedText* conversion : conversions) {
        if (!conversion->value) {
            RecordTechnicalError("ExecuteProce: " + conversion->error);
            return false;
        }
    }

    // ADO/COM is Windows-only plumbing. The existing Linux server stack uses
    // unixODBC with Microsoft ODBC Driver 18; game semantics/procedure contract
    // remain in this original CGame owner.
    std::string connection = "DRIVER={ODBC Driver 18 for SQL Server};SERVER=";
    connection += *host.value;
    connection += ";DATABASE=";
    connection += *database.value;
    connection += ";UID=";
    connection += *dbUser.value;
    connection += ";PWD=";
    connection += *dbPassword.value;
    connection += ";Encrypt=no;TrustServerCertificate=yes;";

    struct Environment
    {
        SQLHENV handle{SQL_NULL_HENV};
        ~Environment()
        {
            if (handle != SQL_NULL_HENV) {
                SQLFreeHandle(SQL_HANDLE_ENV, handle);
            }
        }
    } environment;
    struct Connection
    {
        SQLHDBC handle{SQL_NULL_HDBC};
        ~Connection()
        {
            if (handle != SQL_NULL_HDBC) {
                SQLDisconnect(handle);
                SQLFreeHandle(SQL_HANDLE_DBC, handle);
            }
        }
    } db;
    struct Statement
    {
        SQLHSTMT handle{SQL_NULL_HSTMT};
        ~Statement()
        {
            if (handle != SQL_NULL_HSTMT) {
                SQLFreeHandle(SQL_HANDLE_STMT, handle);
            }
        }
    } statement;

    SQLRETURN result =
        SQLAllocHandle(SQL_HANDLE_ENV, SQL_NULL_HANDLE, &environment.handle);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError("ExecuteProce: SQLAllocHandle(ENV) failed");
        return false;
    }
    result = SQLSetEnvAttr(environment.handle,
                           SQL_ATTR_ODBC_VERSION,
                           reinterpret_cast<SQLPOINTER>(SQL_OV_ODBC3),
                           0);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_ENV, environment.handle, "SQLSetEnvAttr"));
        return false;
    }
    result = SQLAllocHandle(SQL_HANDLE_DBC, environment.handle, &db.handle);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_ENV, environment.handle, "SQLAllocHandle(DBC)"));
        return false;
    }
    result = SQLDriverConnect(
        db.handle,
        nullptr,
        reinterpret_cast<SQLCHAR*>(connection.data()),
        SQL_NTS,
        nullptr,
        0,
        nullptr,
        SQL_DRIVER_NOPROMPT);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_DBC, db.handle, "SQLDriverConnect"));
        return false;
    }
    result = SQLAllocHandle(SQL_HANDLE_STMT, db.handle, &statement.handle);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_DBC, db.handle, "SQLAllocHandle(STMT)"));
        return false;
    }

    // Original ADODB CommandType=4 (stored procedure), CommandText=getAccInfoEx.
    SQLCHAR procedure[] = "{CALL getAccInfoEx(?,?,?,?)}";
    result = SQLPrepare(statement.handle, procedure, SQL_NTS);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_STMT, statement.handle, "SQLPrepare(getAccInfoEx)"));
        return false;
    }

    std::string parameterUser = *convertedUser.value;
    std::string parameterIp = *convertedIp.value;
    std::string parameterPassword = *convertedPassword.value;
    SQLLEN userLength = static_cast<SQLLEN>(parameterUser.size());
    SQLLEN ipLength = static_cast<SQLLEN>(parameterIp.size());
    SQLLEN passwordLength = static_cast<SQLLEN>(parameterPassword.size());
    SQLINTEGER procedureResult = 0;
    SQLLEN resultLength = 0;

    const auto bindText = [&](SQLUSMALLINT number,
                              std::string& value,
                              SQLULEN legacySize,
                              SQLLEN& length) -> bool {
        const SQLRETURN bound = SQLBindParameter(
            statement.handle,
            number,
            SQL_PARAM_INPUT,
            SQL_C_CHAR,
            SQL_VARCHAR,
            legacySize,
            0,
            value.data(),
            static_cast<SQLLEN>(value.size() + 1U),
            &length);
        if (!OdbcSucceeded(bound)) {
            RecordTechnicalError(
                "ExecuteProce: " +
                OdbcDiagnostic(SQL_HANDLE_STMT, statement.handle, "SQLBindParameter"));
            return false;
        }
        return true;
    };

    // VERIFIED_ASSEMBLY: @UserID adVarChar input size 0x20,
    // @UserIP size 0x18, @UserPwd size 0x40, затем @Result adInteger OUTPUT.
    if (!bindText(1, parameterUser, 0x20U, userLength) ||
        !bindText(2, parameterIp, 0x18U, ipLength) ||
        !bindText(3, parameterPassword, 0x40U, passwordLength)) {
        return false;
    }
    result = SQLBindParameter(statement.handle,
                              4,
                              SQL_PARAM_OUTPUT,
                              SQL_C_SLONG,
                              SQL_INTEGER,
                              0,
                              0,
                              &procedureResult,
                              sizeof(procedureResult),
                              &resultLength);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_STMT, statement.handle, "SQLBindParameter(@Result)"));
        return false;
    }

    result = SQLExecute(statement.handle);
    if (!OdbcSucceeded(result) && result != SQL_NO_DATA) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_STMT, statement.handle, "SQLExecute(getAccInfoEx)"));
        return false;
    }

    // Direct EXE не читает output @Result и на success тоже делает xor al,al.
    static_cast<void>(procedureResult);
    return false;
}

void CGame::RecordTechnicalError(std::string detail)
{
    std::lock_guard guard(m_TechnicalErrorMutex);
    m_TechnicalErrors.push_back(std::move(detail));
}

std::optional<std::string> CGame::PopTechnicalError()
{
    std::lock_guard guard(m_TechnicalErrorMutex);
    if (m_TechnicalErrors.empty()) {
        return std::nullopt;
    }
    std::string detail = std::move(m_TechnicalErrors.front());
    m_TechnicalErrors.pop_front();
    return detail;
}
}
