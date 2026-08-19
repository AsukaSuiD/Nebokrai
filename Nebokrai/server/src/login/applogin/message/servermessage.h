#pragma once

#include "../../servlogqueue.h"
#include "../../../nets/netlogin/message.h"

#include <cstdint>
#include <optional>
#include <span>
#include <vector>

/*
 * Owner: loginserver/applogin/message/servermessage.cpp
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Исходный путь PDB:
 * d:\\complite_version\\fengyun_russia\\trunk\\server\\loginserver\\applogin\\message\\servermessage.cpp
 * OnServerMessage RVA 0x00080850.
 *
 * Восстановлены все достигнутые case: 0x1FE01/0xFF01 — World lifecycle,
 * 0x1FE02/0x1FE03 — snapshot/clear CD-key, 0x1FE04 — World/Game telemetry,
 * 0x1FE05/0x1FE06/0x1FE08 — очередь server-info log. Неизвестный opcode
 * остаётся исходным no-op.
 *
 * Существенный порядок connect: world ID/name -> numeric socket identity ->
 * CGame::AddWorld -> operator log -> 0x4FC03(area_id) тому же socket ->
 * условный _serv_logs. Rejected AddWorld и неуспешный ack не отменяют более
 * поздние позиции. Disconnect берёт World ID из message metadata, для
 * известного имени пишет lost-log и чистит его CD-key, затем всегда DelWorld.
 *
 * 0x1FE02 сохраняет исходный short-circuit: world_id==0 не читает count;
 * count==0 не запускает DB update; отрицательный ненулевой count не читает
 * строки, но всё равно передаёт пустой snapshot DB-owner. Для положительного
 * count каждый account сначала идёт в AddCdkey и независимо попадает в DB
 * snapshot. Никакие защитные capacity-лимиты старого Linux-донора сюда не
 * перенесены: точный owner их не имел.
 *
 * 0x1FE04 хранит 32-битные bit-pattern player/port полей и dotted IPv4 из
 * исходного little-endian m_dwIP. 0x1FE05/06/08 проверяют dwServerInfoLogTime
 * до чтения payload. Connect-строки сохраняют точные CP936 bytes. Формирование
 * локальных date/time отдаётся стандартному strftime вместо собственной
 * реализации CRT-форматирования; два вызова времени остаются раздельными, как
 * исходные _strdate/_strtime.
 *
 * IServerMessageContext заменяет только старый глобальный GetGame() и содержит
 * ровно вызовы CGame, достигнутые этим owner-ом. Сетевой I/O, DB worker и FIFO
 * остаются у своих владельцев; handler не реализует их повторно.
 */
namespace Login
{
struct PingGameServerInfo
{
    std::vector<std::uint8_t> ip;
    std::uint32_t port{};
    std::uint32_t playerCount{};
};

struct PingWorldServerInfo
{
    std::vector<std::uint8_t> ip;
    std::uint32_t port{};
    std::uint32_t playerCount{};
    std::vector<PingGameServerInfo> gameServers;
};

class IServerMessageContext
{
public:
    virtual ~IServerMessageContext() = default;

    virtual void SetWorldSocketMapID(std::int32_t socketId,
                                     std::int32_t worldId) = 0;
    [[nodiscard]] virtual std::int32_t
    AddWorld(std::int32_t worldId, std::span<const std::uint8_t> worldName) = 0;
    virtual void QueueWorldConnectedOperatorLog(
        std::span<const std::uint8_t> worldName) = 0;
    virtual void SendToWorldSocket(const LoginNet::CMessage& message,
                                   std::int32_t socketId) = 0;
    [[nodiscard]] virtual std::int32_t AreaID() const noexcept = 0;

    [[nodiscard]] virtual std::optional<std::vector<std::uint8_t>>
    WorldNameByID(std::int32_t worldId) const = 0;
    virtual void QueueWorldLostOperatorLog(
        std::span<const std::uint8_t> worldName) = 0;
    virtual void ClearCdkeysByWorldID(std::int32_t worldId) = 0;
    [[nodiscard]] virtual std::int32_t DelWorld(std::int32_t worldId) = 0;

    [[nodiscard]] virtual bool AddCdkey(std::span<const std::uint8_t> account,
                                        std::int32_t worldId) = 0;
    virtual void QueueOnlineUserDatabaseUpdate(
        std::int32_t worldId,
        std::vector<std::vector<std::uint8_t>> accounts) = 0;
    virtual void ClearCdkey(std::span<const std::uint8_t> account) = 0;

    virtual void AppendPingWorldServerInfo(PingWorldServerInfo info) = 0;

    // None существует только до материализации LoadSetup в частичном CGame;
    // в исходном post-Init handler значение всегда было обычным DWORD.
    [[nodiscard]] virtual std::optional<std::uint32_t>
    ServerInfoLogTime() const noexcept = 0;
    virtual void PushServerInfoLog(ServLog record) = 0;
};

class ServerMessageHandler
{
public:
    explicit ServerMessageHandler(IServerMessageContext& context) noexcept;

    void OnServerMessage(LoginNet::CMessage& message);

private:
    void OnWorldConnected(LoginNet::CMessage& message);
    void OnWorldDisconnected(const LoginNet::CMessage& message);
    void OnWorldUsersSnapshot(LoginNet::CMessage& message);
    void OnAccountsCleared(LoginNet::CMessage& message);
    void OnWorldTelemetry(LoginNet::CMessage& message);
    void OnGameServerConnectedLog(LoginNet::CMessage& message);
    void OnWorldServerLog(LoginNet::CMessage& message);
    void OnTypedServerLog(LoginNet::CMessage& message);
    void OnSuppliedServerLog(LoginNet::CMessage& message,
                             std::int32_t serverType);

    [[nodiscard]] bool ServerInfoLogEnabled() const noexcept;

    IServerMessageContext& m_Context;
};
}
