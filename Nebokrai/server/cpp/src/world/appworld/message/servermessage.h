#pragma once

#include "../../../nets/networld/message.h"

#include <array>
#include <cstddef>
#include <functional>

/*
 * Исходные владельцы: WorldServer/appworld/message/*.cpp/.h. nets::CMessage
 * уже сохраняет доказанную маршрутизацию 13 семейств. Здесь старые global
 * On* функции заменены owned таблицей обработчиков; payload и read cursor не
 * преобразуются, поэтому точный разбор остаётся у игрового owner-а.
 */
enum class WorldMessageFamily : std::size_t {
    Server, Log, Gma, Player, Other, Gm, Team, Organizing, WriteLog,
    Country, ServerAuction, Jjc, MiscAuction, Count
};

class WorldMessageHandlers final : public WorldNet::IWorldMessageHandlers {
public:
    using Handler = std::function<void(WorldNet::CMessage&)>;
    void Set(WorldMessageFamily family, Handler handler);
    void EnableWriteLog(bool value) noexcept { m_WriteLogEnabled=value; }
    [[nodiscard]] bool WriteLogEnabled() const noexcept override { return m_WriteLogEnabled; }
    void OnServer(WorldNet::CMessage&) override;
    void OnLog(WorldNet::CMessage&) override;
    void OnGma(WorldNet::CMessage&) override;
    void OnPlayer(WorldNet::CMessage&) override;
    void OnOther(WorldNet::CMessage&) override;
    void OnGm(WorldNet::CMessage&) override;
    void OnTeam(WorldNet::CMessage&) override;
    void OnOrganizingSystem(WorldNet::CMessage&) override;
    void OnWriteLog(WorldNet::CMessage&) override;
    void OnCountry(WorldNet::CMessage&) override;
    void OnServerAuction(WorldNet::CMessage&) override;
    void OnJjcSystem(WorldNet::CMessage&) override;
    void OnMiscAuction(WorldNet::CMessage&) override;
    void Dispatch(WorldMessageFamily family, WorldNet::CMessage& message);
private:
    std::array<Handler,static_cast<std::size_t>(WorldMessageFamily::Count)> m_Handlers{};
    bool m_WriteLogEnabled{};
};
