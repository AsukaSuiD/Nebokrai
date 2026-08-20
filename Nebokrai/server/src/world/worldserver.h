#pragma once

#include "game.h"
#include "../dbaccess/worlddb/rssetup.h"
#include "../nets/networld/mynetclient.h"
#include "../nets/networld/mynetserver.h"

#include <asio.hpp>

#include <cstdint>
#include <filesystem>
#include <memory>
#include <optional>
#include <string>
#include <system_error>

/*
 * Исходный владелец: WorldServer/worldserver/worldserver.cpp/.h.
 * WinMain, IOCP threads и глобальный GetGame заменены обычным Linux process-
 * owner-ом. Порядок setup -> DB -> listener -> Login link -> main loop ->
 * final save/release сохранён; Asio и ODBC меняют только инфраструктуру.
 */
enum class WorldServerState { Created, Initialized, Running, Stopping, Stopped, Error };

struct WorldInitializationResult {
    bool success{};
    std::string error;
    [[nodiscard]] explicit operator bool() const noexcept { return success; }
};

struct WorldLoopResult {
    bool success{true};
    std::size_t messages{};
    std::size_t ioActions{};
    bool saved{};
    std::string error;
};

class CWorldServer {
public:
    CWorldServer();
    ~CWorldServer();
    CWorldServer(const CWorldServer&) = delete;
    CWorldServer& operator=(const CWorldServer&) = delete;

    [[nodiscard]] WorldInitializationResult Initialize(const std::filesystem::path& runtimeDirectory);
    [[nodiscard]] WorldLoopResult RunOne();
    void RequestStop() noexcept;
    [[nodiscard]] bool Shutdown();
    [[nodiscard]] WorldServerState State() const noexcept { return m_State; }
    [[nodiscard]] CGame* Game() noexcept { return m_Game.get(); }

private:
    [[nodiscard]] static std::uint32_t Now() noexcept;
    [[nodiscard]] asio::awaitable<void> AcceptLoop();
    [[nodiscard]] asio::awaitable<void> LoginReadLoop(asio::ip::tcp::endpoint endpoint);
    [[nodiscard]] asio::awaitable<void> FlushLogin();
    [[nodiscard]] bool LoadPersistentOwners(std::string& error);
    void InstallMessageHandlers();
    void HandleServerMessage(WorldNet::CMessage& message);
    void HandlePlayerMessage(WorldNet::CMessage& message);
    void HandleCountryMessage(WorldNet::CMessage& message);
    void HandleTeamMessage(WorldNet::CMessage& message);
    void HandleJjcMessage(WorldNet::CMessage& message);
    void HandleUnhandledMessage(std::string_view family, WorldNet::CMessage& message);
    [[nodiscard]] bool SendInitialGameServerState(std::int32_t socketId,
                                                  const CGame::GameServerInfo& gameServer);
    void QueueServerIo(ServerSnapshot snapshot, std::size_t& count);

    asio::io_context m_Io;
    std::unique_ptr<WorldNet::CMyNetServer> m_Server;
    std::unique_ptr<WorldNet::CMyNetClient> m_Login;
    std::unique_ptr<OdbcWorldDbExecutor> m_Database;
    std::unique_ptr<CGame> m_Game;
    std::optional<WorldSaveBatch> m_PendingSave;
    WorldServerState m_State{WorldServerState::Created};
    std::uint32_t m_LastSaveTick{};
    std::uint32_t m_LoginReconnectAt{};
    std::optional<asio::ip::tcp::endpoint> m_LoginEndpoint;
    bool m_LoginLoopActive{};
    bool m_LoginFlushActive{};
    std::string m_AsyncError;
};
