#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <deque>
#include <mutex>
#include <variant>
#include <vector>

/*
 * Исходный владелец: authserver/src/dbqueue.h
 *
 * Точная пара: AuthServer/authserver.exe + AuthServer/authserver.pdb; исходный
 * Путь владельца в PDB: h:\fengyun\fy_russia\src\server\authserver\src\dbqueue.h.
 * Поздняя реконструкция подтверждает формы Auth DB quest/result и
 * специальную coalescing-очередь ServerInfo. Старый db_element_type связывал
 * integer tag с void*; новый C++ использует std::variant и владеющие значения,
 * поэтому неверная пара tag/payload и ручной delete больше невозможны.
 *
 * ServerInfoQueue сохраняет отдельную оригинальную семантику: ключ
 * (login, world, game) обновляет только player count на прежней позиции;
 * новый ключ добавляется в хвост; PopAll атомарно передаёт весь snapshot.
 */
namespace AuthDb
{
struct AuthQuestData
{
    std::vector<std::uint8_t> account;
    std::vector<std::uint8_t> password;
    std::uint32_t clientIp{};
    std::int32_t clientSocketId{};
};

struct LockUntil
{
    std::uint16_t year{};
    std::uint16_t month{};
    std::uint16_t day{};
    std::uint16_t hour{};
    std::uint16_t minute{};
    std::uint16_t second{};
};

struct LockQuestData
{
    std::vector<std::uint8_t> account;
    LockUntil until;
};

struct AuthenticateQuest
{
    std::int32_t returnSocketId{};
    AuthQuestData request;
};

struct AuthenticateExtendedQuest
{
    std::int32_t returnSocketId{};
    AuthQuestData request;
};

struct LockQuest
{
    std::int32_t returnSocketId{};
    LockQuestData request;
};

struct WriteServerInfoQuest {};

using DbQuest = std::variant<AuthenticateQuest,
                             AuthenticateExtendedQuest,
                             LockQuest,
                             WriteServerInfoQuest>;

struct AuthResultData
{
    std::int32_t result{};
    std::vector<std::uint8_t> account;
    std::uint32_t clientIp{};
    std::int32_t clientSocketId{};
};

struct AuthExResultData
{
    std::int32_t result{};
    std::vector<std::uint8_t> account;
    std::uint32_t clientIp{};
    std::int32_t clientSocketId{};
    std::array<std::uint8_t, 80> extra{};
};

struct LockResultData
{
    std::vector<std::uint8_t> account;
    bool succeeded{};
};

struct AuthenticateResult
{
    std::int32_t returnSocketId{};
    AuthResultData result;
};

struct AuthenticateExtendedResult
{
    std::int32_t returnSocketId{};
    AuthExResultData result;
};

struct LockResult
{
    std::int32_t returnSocketId{};
    LockResultData result;
};

using DbResult = std::variant<AuthenticateResult,
                              AuthenticateExtendedResult,
                              LockResult>;

struct ServerInfo
{
    std::int32_t playerCount{};
    std::int32_t gameServerId{};
    std::int32_t worldServerId{};
    std::int32_t loginServerId{};

    [[nodiscard]] bool HasSameKey(const ServerInfo& other) const noexcept;
};

class ServerInfoQueue
{
public:
    void PushBack(ServerInfo entry);
    [[nodiscard]] std::deque<ServerInfo> PopAll();
    [[nodiscard]] std::size_t Size() const;

private:
    mutable std::mutex m_Mutex;
    std::deque<ServerInfo> m_Entries;
};
}
