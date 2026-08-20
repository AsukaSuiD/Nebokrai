#pragma once

#include "../baseobject.h"

#include <cstddef>
#include <cstdint>
#include <span>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/session/csession.cpp / csession.h.
 * Minimum/maximum plugs, lifetime, start/end/abort и ordered plug broadcast
 * подтверждены PDB/Rust factory owner-ом. timeGetTime заменён monotonic clock,
 * std::vector хранит тот же insertion order; фабрика остаётся единственным
 * владельцем plug-ов.
 */
class CPlug;
class CSessionFactory;

class CSession : public CBaseObject
{
public:
    CSession(CSessionFactory& factory,
             std::uint32_t minimumPlugs,
             std::uint32_t maximumPlugs,
             std::uint32_t lifetimeMs) noexcept;
    ~CSession() override = default;

    [[nodiscard]] virtual bool IsSessionEnded() const noexcept { return m_Started && m_Ended; }
    [[nodiscard]] virtual bool IsSessionAvailable() const;
    [[nodiscard]] CPlug* QueryPlugByOwner(std::int32_t ownerType, std::int32_t ownerId) const noexcept;
    [[nodiscard]] CPlug* QueryPlugByID(std::int32_t plugId) const noexcept;
    virtual bool InsertPlug(std::int32_t plugId);
    virtual bool Start();
    virtual bool End();
    virtual bool Abort();
    virtual bool OnSessionStarted() { return true; }
    virtual bool OnSessionEnded() { return true; }
    virtual bool OnSessionAborted() { return true; }
    virtual bool OnPlugInserted(std::int32_t plugId);
    virtual bool OnPlugEnded(std::int32_t plugId);
    virtual bool OnPlugAborted(std::int32_t plugId);
    virtual bool OnPlugChangeState(std::int32_t plugId,
                                   std::int32_t state,
                                   std::span<const std::uint8_t> payload,
                                   bool includeSender);
    virtual bool Serialize(std::vector<std::uint8_t>& output) const;
    virtual bool Unserialize(std::span<const std::uint8_t>, std::size_t&) { return true; }
    void AI() override;

    [[nodiscard]] const std::vector<std::int32_t>& Plugs() const noexcept { return m_Plugs; }
    [[nodiscard]] std::uint32_t GetSessionType() const noexcept { return m_SessionType; }

protected:
    void SetSessionType(std::uint32_t value) noexcept { m_SessionType = value; }
    CSessionFactory& Factory() const noexcept { return m_Factory; }
    [[nodiscard]] bool Started() const noexcept { return m_Started; }

private:
    static std::uint32_t Now() noexcept;
    CSessionFactory& m_Factory;
    std::uint32_t m_SessionType{};
    bool m_Ended{};
    bool m_Started{};
    bool m_Aborted{};
    std::uint32_t m_MaximumPlugs{};
    std::uint32_t m_MinimumPlugs{};
    std::uint32_t m_StartedAt{};
    std::uint32_t m_LifetimeMs{};
    std::vector<std::int32_t> m_Plugs;
};
