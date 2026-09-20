#pragma once

#include "../../../nets/mysocket.h"
#include "../../../nets/netauth/message.h"
#include "../../../nets/servers.h"
#include "../../dbqueue.h"
#include "../../kl_ipfilter.h"

#include <cstdint>
#include <deque>
#include <functional>
#include <map>
#include <optional>
#include <variant>
#include <vector>

/*
 * Исходный владелец: authserver/appauth/message/message_func.cpp
 *
 * Все девять функций Auth InitMsgFuncPool восстановлены по точной паре
 * authserver.exe/authserver.pdb. Подтверждённые RVA: отключение через GM
 * 0x00016010, ответ на отключение 0x00016220, блокировка через GM 0x000163F0,
 * обычная/расширенная авторизация 0x00016550/0x00016720, подключение,
 * отключение и запрос сведений LS 0x00016900/0x00016AD0/0x00016BD0,
 * ответ со сведениями о сервере 0x00016CD0.
 *
 * Здесь сохраняется доменная маршрутизация LoginServer: area->socket map,
 * IP allow-filter, создание DB quest, coalesced ServerInfo и GM forwarding.
 * Старые MFC ListBox/AddLogText side effects представлены структурированными
 * уведомления; владелец журнала Linux сможет вывести их позже без зависимости от GUI.
 *
 * CGame остаётся фактическим владельцем очередей БД. Чтобы этот владелец исходника не
 * тянул полувосстановленный глобальный singleton, два доказанных действия
 * передаются callback-ами: PushDBQuest и PushServerInfo. Это граница вызова, а
 * не новый долговечный владелец или архитектура.
 */

struct LoginServerConnectedNotice
{
    IPv4Octets address{};
    std::int32_t socketId{};
    bool allowed{};
};

struct LoginServerRegisteredNotice
{
    IPv4Octets address{};
    std::int32_t areaId{};
    std::int32_t socketId{};
};

struct LoginServerDisconnectedNotice
{
    IPv4Octets address{};
    std::int32_t areaId{};
    std::int32_t socketId{};
};

using LoginServerNotice = std::variant<LoginServerConnectedNotice,
                                       LoginServerRegisteredNotice,
                                       LoginServerDisconnectedNotice>;

class AuthMessageHandlers final : public IAuthMessageHandler
{
public:
    using DbQuestSink = std::function<bool(AuthDb::DbQuest)>;
    using ServerInfoSink = std::function<void(AuthDb::ServerInfo)>;

    AuthMessageHandlers(bool ipFilterEnabled,
                        std::vector<kl_net::IpPattern> allowedPatterns,
                        DbQuestSink pushQuest,
                        ServerInfoSink pushServerInfo);

    void ReplaceIpFilter(bool enabled, std::vector<kl_net::IpPattern> allowedPatterns);

    [[nodiscard]] std::int32_t LoginServerSocketID(std::int32_t areaId) const noexcept;
    [[nodiscard]] std::int32_t LoginServerAreaID(std::int32_t socketId) const noexcept;
    [[nodiscard]] std::vector<std::int32_t> LoginServerSocketIDs() const;
    [[nodiscard]] std::optional<LoginServerNotice> PopNotice();

    [[nodiscard]] AuthDispatchStatus Handle(AuthMessageKind kind,
                                            CMessage& message,
                                            const ServerCommandHandle& sender) override;

private:
    void OnLoginServerConnect(const CMessage& message, const ServerCommandHandle& sender);
    void OnLoginServerDisconnect(const CMessage& message);
    void OnLoginServerGetInfo(CMessage& message);
    void OnUpdateServerInfoResponse(CMessage& message);
    void OnAuthAccount(CMessage& message, bool extended);
    void OnGmLockAccount(CMessage& message);
    [[nodiscard]] AuthDispatchStatus OnGmKickPlayer(CMessage& message,
                                                    const ServerCommandHandle& sender) const;
    [[nodiscard]] AuthDispatchStatus OnKickPlayerResponse(CMessage& message,
                                                          const ServerCommandHandle& sender) const;
    [[nodiscard]] std::optional<std::int32_t> AreaForSocket(std::int32_t socketId) const noexcept;

    std::map<std::int32_t, std::int32_t> m_LoginServers;
    bool m_IpFilterEnabled{};
    kl_net::IpFilter<true> m_IpAllower;
    DbQuestSink m_PushQuest;
    ServerInfoSink m_PushServerInfo;
    std::deque<LoginServerNotice> m_Notices;
};
