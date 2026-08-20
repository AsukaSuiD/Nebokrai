#pragma once

#include <cstdint>
#include <filesystem>
#include <map>
#include <optional>
#include <string>
#include <string_view>

/*
 * Исходный владелец: authserver/src/configreader.cpp / configreader.h
 *
 * Точная пара: AuthServer/authserver.exe + AuthServer/authserver.pdb.
 * Путь владельца в PDB: h:\fengyun\fy_russia\src\server\authserver\src\configreader.cpp
 * Подтверждённые RVA: getDBSP 0x00009560, reset 0x0000A380,
 * set_sp_name 0x0000A940, конструктор 0x0000AA60, load 0x0000AB60.
 *
 * setup.ini здесь не является обычным INI. Оригинал двадцать раз читает
 * whitespace-token label, не проверяет его текст, затем значение фиксированного
 * типа. Поэтому порядок пар определяет назначение, а хвост после двадцатой пары
 * игнорируется. Универсальный INI parser намеренно не используется.
 *
 * После успешного открытия parse-error оставляет уже записанный prefix полей;
 * затем оригинал всё равно принудительно выключает EnableIPFilter, записывает
 * четыре имени stored procedures и возвращает false. Ошибка открытия объект не
 * меняет. Malformed numeric MSVC num_get остаётся локальной неизвестностью;
 * безопасный parser принимает только полностью корректный decimal baseline.
 */

enum class ConfigLoadStatus
{
    Ok,
    OpenFailed,
    MissingToken,
    InvalidValue,
};

struct ConfigLoadResult
{
    ConfigLoadStatus status{ConfigLoadStatus::OpenFailed};
    const char* field{};

    [[nodiscard]] constexpr explicit operator bool() const noexcept
    {
        return status == ConfigLoadStatus::Ok;
    }
};

struct AuthNetworkConfig
{
    std::uint32_t hostPort{};
    std::int32_t maxLoginServers{};
    std::int32_t maxInFlightSends{};
    std::int32_t permittedSendBytes{};
    std::int32_t newAcceptTimeoutMs{};
};

struct AuthDatabaseSettings
{
    std::string host;
    std::string database;
    std::string user;
    std::string password;
};

class ConfigReader
{
public:
    ConfigReader();

    void Reset();
    [[nodiscard]] ConfigLoadResult Load(const std::filesystem::path& path);

    [[nodiscard]] std::string_view GetDBSP(std::string_view identifier) const noexcept;

    [[nodiscard]] std::int32_t MaxAuthQueueSize() const noexcept;
    [[nodiscard]] std::int32_t DatabaseThreadCount() const noexcept;
    [[nodiscard]] bool ClientIpFilterEnabled() const noexcept;
    [[nodiscard]] bool LoginServerIpFilterEnabled() const noexcept;
    [[nodiscard]] bool UpdateServerInfoEnabled() const noexcept;
    [[nodiscard]] std::uint32_t UpdateServerInfoTimeMs() const noexcept;
    [[nodiscard]] std::uint32_t WriteServerInfoTimeMs() const noexcept;
    [[nodiscard]] AuthNetworkConfig NetworkConfig() const noexcept;
    [[nodiscard]] AuthDatabaseSettings AuthDatabase() const;
    [[nodiscard]] AuthDatabaseSettings LogDatabase() const;

private:
    void SetStoredProcedureNames();

    std::uint32_t m_HostPort{};
    std::int32_t m_DbThreadCount{};
    std::int32_t m_MaxLoginServerCount{};
    std::int32_t m_SendIoNumber{};
    std::int32_t m_MaxClientSendBufferSize{};
    std::int32_t m_SendInterTime{};
    bool m_EnableIpFilter{};
    std::string m_DbIp;
    std::string m_AuthDatabaseName;
    std::string m_DbUser;
    std::string m_DbPassword;
    std::string m_DbSpConfig;
    std::string m_LogDbIp;
    std::string m_LogDatabaseName;
    std::string m_LogDbUser;
    std::string m_LogDbPassword;
    std::int32_t m_MaxAuthQueueSize{};
    bool m_EnableClientIpFilter{};
    std::uint32_t m_UpdateServerInfoTimeMs{};
    std::uint32_t m_WriteServerInfoTimeMs{};
    bool m_EnableUpdateServerInfo{};
    std::map<std::string, std::string, std::less<>> m_DbStoredProcedures;
};
