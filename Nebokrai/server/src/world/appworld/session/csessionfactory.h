#pragma once

#include <cstddef>
#include <cstdint>
#include <map>
#include <memory>
#include <span>

/*
 * Исходный владелец: WorldServer/appworld/session/csessionfactory.cpp/.h.
 * Два registry, initial ID=1, типы 10/11, factory choices и AI erase-order
 * подтверждены exact EXE/PDB и полностью достигнуты Rust-реконструкцией.
 * MSVC hash internals удалены; std::map даёт стабильный signed-key traversal,
 * unique_ptr сохраняет factory-owned lifetime без deleting thunks.
 */
class CPlug;
class CSession;

class CSessionFactory
{
public:
    enum class SessionType : std::int32_t { Normal = 0, Team = 1 };
    enum class PlugType : std::int32_t {
        Normal = 0, Trader = 1, PersonalShopSeller = 2,
        PersonalShopBuyer = 3, EquipmentUpgrade = 4, Teamate = 5,
    };
    enum class ObjectType : std::int32_t { Session = 10, Plug = 11 };

    [[nodiscard]] CSession* QuerySession(std::int32_t id) const noexcept;
    [[nodiscard]] CPlug* QueryPlug(std::int32_t id) const noexcept;
    [[nodiscard]] std::int32_t CreateSession(std::uint32_t minimumPlugs,
                                             std::uint32_t maximumPlugs,
                                             std::uint32_t lifetimeMs,
                                             SessionType type);
    [[nodiscard]] std::int32_t CreatePlug(PlugType type,
                                          std::int32_t ownerType,
                                          std::int32_t ownerId);
    bool InsertPlug(std::int32_t sessionId, std::int32_t plugId);
    bool GarbageCollect(ObjectType type, std::int32_t id);
    void AI();
    [[nodiscard]] std::int32_t UnserializeSession(std::span<const std::uint8_t> input,
                                                  std::size_t& offset);
    [[nodiscard]] std::int32_t UnserializePlug(std::span<const std::uint8_t> input,
                                               std::size_t& offset);

private:
    std::int32_t m_NextSessionId{1};
    std::int32_t m_NextPlugId{1};
    std::map<std::int32_t, std::unique_ptr<CSession>> m_Sessions;
    std::map<std::int32_t, std::unique_ptr<CPlug>> m_Plugs;
};
