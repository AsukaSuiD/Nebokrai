#pragma once

#include "../baseobject.h"

#include <cstddef>
#include <cstdint>
#include <span>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/session/cplug.cpp / cplug.h.
 * Owner/session identity, ended lifecycle и wire header подтверждены PDB и
 * псевдокодом. Глобальный CSessionFactory заменён явной ссылкой; raw payload
 * получает размер через span. Конкретные игровые реакции принадлежат derived
 * plug-ам и не подменяются заглушками фабрики.
 */
class CSession;
class CSessionFactory;
class CContainer;

class CPlug : public CBaseObject
{
public:
    explicit CPlug(CSessionFactory& factory) noexcept;
    ~CPlug() override = default;

    void SetOwner(std::int32_t type, std::int32_t id) noexcept;
    void SetSession(std::int32_t id) noexcept { m_SessionId = id; }
    [[nodiscard]] CSession* GetSession() const noexcept;
    [[nodiscard]] std::int32_t GetOwnerType() const noexcept { return m_OwnerType; }
    [[nodiscard]] std::int32_t GetOwnerID() const noexcept { return m_OwnerId; }
    [[nodiscard]] std::int32_t GetSessionID() const noexcept { return m_SessionId; }
    [[nodiscard]] std::uint32_t GetPlugType() const noexcept { return m_PlugType; }

    virtual bool ChangeState(std::int32_t state, std::span<const std::uint8_t> payload = {});
    virtual bool Exit();
    [[nodiscard]] virtual bool IsPlugAvailable() const { return true; }
    virtual bool OnSessionEnded() { return true; }
    virtual bool OnSessionAborted() { return true; }
    virtual bool OnChangeState(std::int32_t, std::int32_t, std::span<const std::uint8_t>) { return true; }
    virtual bool OnPlugInserted() { return true; }
    virtual bool OnPlugAborted() { return true; }
    virtual bool OnPlugEnded() { return true; }
    [[nodiscard]] virtual bool IsPlugEnded() const noexcept { return m_Ended; }
    virtual CContainer* GetContainer(std::int32_t) noexcept { return nullptr; }
    virtual bool Serialize(std::vector<std::uint8_t>& output) const;
    virtual bool Unserialize(std::span<const std::uint8_t> input, std::size_t& offset);

protected:
    void SetPlugType(std::uint32_t value) noexcept { m_PlugType = value; }
    CSessionFactory& Factory() const noexcept { return m_Factory; }

private:
    CSessionFactory& m_Factory;
    std::int32_t m_SessionId{};
    std::int32_t m_OwnerType{};
    std::int32_t m_OwnerId{};
    std::uint32_t m_PlugType{};
    bool m_Ended{};
};
