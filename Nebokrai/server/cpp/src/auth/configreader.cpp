#include "configreader.h"

#include <charconv>
#include <fstream>
#include <limits>
#include <string>
#include <type_traits>

namespace
{
class TokenReader
{
public:
    explicit TokenReader(std::istream& input)
        : m_Input(input)
    {
    }

    template <typename T>
    ConfigLoadResult ReadNumber(const char* field, T& destination)
    {
        std::string label;
        std::string value;
        if (!(m_Input >> label) || !(m_Input >> value)) {
            return {ConfigLoadStatus::MissingToken, field};
        }

        T parsed{};
        const char* first = value.data();
        const char* last = first + value.size();
        const auto [end, error] = std::from_chars(first, last, parsed, 10);
        if (error != std::errc{} || end != last) {
            return {ConfigLoadStatus::InvalidValue, field};
        }
        destination = parsed;
        return {ConfigLoadStatus::Ok, nullptr};
    }

    ConfigLoadResult ReadBool(const char* field, bool& destination)
    {
        unsigned int parsed{};
        const ConfigLoadResult result = ReadNumber(field, parsed);
        if (!result) {
            return result;
        }
        if (parsed > 1U) {
            return {ConfigLoadStatus::InvalidValue, field};
        }
        destination = parsed != 0U;
        return {ConfigLoadStatus::Ok, nullptr};
    }

    ConfigLoadResult ReadBytes(const char* field, std::string& destination)
    {
        std::string label;
        std::string value;
        if (!(m_Input >> label) || !(m_Input >> value)) {
            return {ConfigLoadStatus::MissingToken, field};
        }
        destination = std::move(value);
        return {ConfigLoadStatus::Ok, nullptr};
    }

private:
    std::istream& m_Input;
};
}

ConfigReader::ConfigReader()
{
    Reset();
}

void ConfigReader::Reset()
{
    m_HostPort = 0x1BBCU;
    m_DbThreadCount = 1;
    m_MaxLoginServerCount = 20;
    m_SendIoNumber = 100;
    m_MaxClientSendBufferSize = 0x0A00'0000;
    m_SendInterTime = 5000;
    m_EnableIpFilter = false;
    m_EnableClientIpFilter = false;
    m_UpdateServerInfoTimeMs = 30'000U;
    m_WriteServerInfoTimeMs = 30'000U;
    m_EnableUpdateServerInfo = false;
    m_DbIp.clear();
    m_AuthDatabaseName = "DB_gCFY";
    m_DbUser.clear();
    m_DbPassword.clear();
    m_DbSpConfig = "dbspcfg.ini";
    m_LogDbIp.clear();
    m_LogDatabaseName = "Unknown";
    m_LogDbUser.clear();
    m_LogDbPassword.clear();
    m_MaxAuthQueueSize = 3000;
    m_DbStoredProcedures.clear();
}

ConfigLoadResult ConfigReader::Load(const std::filesystem::path& path)
{
    std::ifstream input(path, std::ios::binary);
    if (!input) {
        return {ConfigLoadStatus::OpenFailed, nullptr};
    }

    TokenReader tokens(input);
    ConfigLoadResult result{ConfigLoadStatus::Ok, nullptr};

#define READ_FIELD(call)            \
    do {                            \
        result = (call);            \
        if (!result) {              \
            goto parse_finished;    \
        }                           \
    } while (false)

    READ_FIELD(tokens.ReadNumber("AuthServerPort", m_HostPort));
    READ_FIELD(tokens.ReadNumber("DatabaseAuthThreadCount", m_DbThreadCount));
    READ_FIELD(tokens.ReadNumber("MaxLoginServerCount", m_MaxLoginServerCount));
    READ_FIELD(tokens.ReadNumber("SendIONum", m_SendIoNumber));
    READ_FIELD(tokens.ReadNumber("MaxClientSendBufSize", m_MaxClientSendBufferSize));
    READ_FIELD(tokens.ReadNumber("SendInterTime", m_SendInterTime));
    READ_FIELD(tokens.ReadBool("EnableIPFilter(NOT_USED)", m_EnableIpFilter));
    READ_FIELD(tokens.ReadBytes("DatabaseIP", m_DbIp));
    READ_FIELD(tokens.ReadBytes("AuthDatabaseName", m_AuthDatabaseName));
    READ_FIELD(tokens.ReadBytes("DatabaseUser", m_DbUser));
    READ_FIELD(tokens.ReadBytes("DatabasePassword", m_DbPassword));
    READ_FIELD(tokens.ReadBytes("LogDatabaseIP", m_LogDbIp));
    READ_FIELD(tokens.ReadBytes("LogDatabaseName", m_LogDatabaseName));
    READ_FIELD(tokens.ReadBytes("LogDatabaseUser", m_LogDbUser));
    READ_FIELD(tokens.ReadBytes("LogDatabasePassword", m_LogDbPassword));
    READ_FIELD(tokens.ReadNumber("MaxAuthQueueSize", m_MaxAuthQueueSize));
    READ_FIELD(tokens.ReadBool("EnableClientIPFilter", m_EnableClientIpFilter));
    READ_FIELD(tokens.ReadNumber("UpdateServerInfoTime(ms)", m_UpdateServerInfoTimeMs));
    READ_FIELD(tokens.ReadNumber("WriteServerInfoTime(ms)", m_WriteServerInfoTimeMs));
    READ_FIELD(tokens.ReadBool("EnableUpdateServerInfo(1or0)", m_EnableUpdateServerInfo));

parse_finished:
#undef READ_FIELD
    // Точная странность Auth RVA 0xAB60: значение из файла отбрасывается и
    // процедуры назначаются даже после failbit уже открытого stream.
    m_EnableIpFilter = false;
    SetStoredProcedureNames();
    return result;
}

std::string_view ConfigReader::GetDBSP(std::string_view identifier) const noexcept
{
    const auto found = m_DbStoredProcedures.find(identifier);
    return found == m_DbStoredProcedures.end() ? std::string_view{} : found->second;
}

std::int32_t ConfigReader::MaxAuthQueueSize() const noexcept
{
    return m_MaxAuthQueueSize;
}

std::int32_t ConfigReader::DatabaseThreadCount() const noexcept
{
    return m_DbThreadCount;
}

bool ConfigReader::ClientIpFilterEnabled() const noexcept
{
    return m_EnableClientIpFilter;
}

bool ConfigReader::LoginServerIpFilterEnabled() const noexcept
{
    return m_EnableIpFilter;
}

bool ConfigReader::UpdateServerInfoEnabled() const noexcept
{
    return m_EnableUpdateServerInfo;
}

std::uint32_t ConfigReader::UpdateServerInfoTimeMs() const noexcept
{
    return m_UpdateServerInfoTimeMs;
}

std::uint32_t ConfigReader::WriteServerInfoTimeMs() const noexcept
{
    return m_WriteServerInfoTimeMs;
}

AuthNetworkConfig ConfigReader::NetworkConfig() const noexcept
{
    return {
        .hostPort = m_HostPort,
        .maxLoginServers = m_MaxLoginServerCount,
        .maxInFlightSends = m_SendIoNumber,
        .permittedSendBytes = m_MaxClientSendBufferSize,
        .newAcceptTimeoutMs = m_SendInterTime,
    };
}

AuthDatabaseSettings ConfigReader::AuthDatabase() const
{
    return {m_DbIp, m_AuthDatabaseName, m_DbUser, m_DbPassword};
}

AuthDatabaseSettings ConfigReader::LogDatabase() const
{
    return {m_LogDbIp, m_LogDatabaseName, m_LogDbUser, m_LogDbPassword};
}

void ConfigReader::SetStoredProcedureNames()
{
    m_DbStoredProcedures["sp_auth"] = "getAccInfo";
    m_DbStoredProcedures["sp_lock"] = "suspendAccount";
    m_DbStoredProcedures["sp_authex"] = "GetAccount";
    m_DbStoredProcedures["sp_writelog"] = "PutOnlineLog";
}
