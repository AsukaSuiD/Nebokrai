#pragma once

#include "setup/setup.h"
#include "../nets/netmisc/message.h"
#include "../nets/netmisc/mynetclient.h"
#include "../public/auctionroom/aucitionroom.h"

#include <asio.hpp>

#include <cstdint>
#include <filesystem>
#include <map>
#include <memory>
#include <optional>
#include <string>

namespace Misc
{
enum class MiscGameStatus
{
    Ok,
    ConfigurationError,
    NetworkError,
    RuntimeError,
};

struct MiscGameResult
{
    MiscGameStatus status{MiscGameStatus::Ok};
    std::string detail;
    [[nodiscard]] explicit operator bool() const noexcept
    {
        return status == MiscGameStatus::Ok;
    }
};

/*
 * Исходный владелец: server/miscserver/miscserver/game.cpp / .h.
 *
 * CGame остаётся владельцем одного World-клиента, комнаты аукциона и исходных
 * turn-local счётчиков. Asio заменяет socket threads; порядок одного turn
 * сохраняется: сеть -> AI -> snapshot сообщений -> reconnect.
 */
class CGame final : private MiscNet::IMiscMessageHandlers
{
public:
    CGame();

    [[nodiscard]] MiscGameResult Initialize(const std::filesystem::path& runtimeDirectory);
    [[nodiscard]] MiscGameResult RunTurn();
    [[nodiscard]] MiscGameResult Release();

    [[nodiscard]] Auction::CAuctionRoom& AuctionRoom() noexcept;
    [[nodiscard]] const Auction::CAuctionRoom& AuctionRoom() const noexcept;
    [[nodiscard]] MiscNet::CMyNetClient* NetClient() noexcept;
    [[nodiscard]] ClientSendQueue* SendQueue() noexcept;
    void CountNewAuctionItem() noexcept;
    [[nodiscard]] std::optional<Auction::AuctionRoomError>
    AddAuctionItem(std::unique_ptr<Auction::CGoodsNode> item, bool& added);
    [[nodiscard]] std::uint32_t AddNewCount() const noexcept;
    [[nodiscard]] std::uint32_t DeletedNewCount() const noexcept;
    [[nodiscard]] bool AuctionSyncEnabled() const noexcept;
    [[nodiscard]] std::uint32_t AuctionSyncStartTime() const noexcept;
    void EnableAuctionSync() noexcept;
    [[nodiscard]] std::optional<std::uint32_t> AuctionSyncCount() const noexcept;
    void FinishAuctionSyncTurn() noexcept;
    void RequestReconnect() noexcept;

private:
    [[nodiscard]] asio::awaitable<MiscGameResult>
    ConnectWorld(std::optional<std::uint16_t> registrationPort,
                 bool sendInitialSync);
    [[nodiscard]] asio::awaitable<MiscGameResult> ProcessMessages();
    void OnWorldAuction(MiscNet::CMessage& message) override;
    void OnMiscFunction(MiscNet::CMessage& message) override;
    void OnOther(MiscNet::CMessage& message) override;

    asio::io_context m_Io;
    CSetup m_Setup;
    Auction::CAuctionRoom m_AuctionRoom;
    std::unique_ptr<MiscNet::CMyNetClient> m_NetClient;
    std::filesystem::path m_RuntimeDirectory;
    bool m_ClientClose{};
    bool m_ReconnectRequested{};
    bool m_DoneSyncMessage{};
    std::uint32_t m_SyncStartTime{};
    std::optional<std::uint32_t> m_DoneSyncCount;
    std::uint32_t m_CurrentMessage{};
    std::map<std::int32_t, std::int32_t> m_MessageRecord;
    std::uint32_t m_LastLogRefreshTime{};
    std::uint32_t m_AddNewCount{};
    std::uint32_t m_DeletedNewCount{};
    bool m_Initialized{};
    bool m_Released{};
};

[[nodiscard]] std::uint32_t LegacyTickMilliseconds() noexcept;
}
