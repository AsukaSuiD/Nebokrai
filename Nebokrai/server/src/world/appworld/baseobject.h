#pragma once

#include "../../public/guid.h"

#include <cstddef>
#include <cstdint>
#include <functional>
#include <list>
#include <memory>
#include <span>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/baseobject.cpp / baseobject.h.
 * Точная пара: Nworldserver.exe + WorldServer.pdb; подтверждённые поля,
 * defaults, wire-порядок и child-tree сверены с поздней Rust-реконструкцией.
 *
 * Сырые owning pointers старого std::list заменены unique_ptr. Наблюдаемая
 * семантика остаётся прежней: parent владеет присоединёнными детьми, обход
 * идёт по snapshot, а извлечение не уничтожает объект. Безразмерный decoder
 * заменён span-границей; повреждённый пакет возвращает false вместо старого UB.
 * Фабрика пяти World-типов определяется после материализации derived owners.
 */
class CBaseObject
{
public:
    using Child = std::unique_ptr<CBaseObject>;
    using ObjectTypeCallback = std::function<void(CBaseObject&)>;

    CBaseObject() = default;
    virtual ~CBaseObject() = default;

    CBaseObject(const CBaseObject&) = delete;
    CBaseObject& operator=(const CBaseObject&) = delete;
    CBaseObject(CBaseObject&&) = delete;
    CBaseObject& operator=(CBaseObject&&) = delete;

    virtual bool Load();
    virtual bool Save();
    virtual bool AddToByteArray(std::vector<std::uint8_t>& output,
                                bool includeChild) const;
    virtual bool DecordFromByteArray(std::span<const std::uint8_t> input,
                                     std::size_t& offset,
                                     bool includeChild);
    virtual void AI();
    virtual void BoardCast(std::int32_t messageId, std::int32_t parameter);

    void FindObjectsByType(std::int32_t type, const ObjectTypeCallback& callback);
    [[nodiscard]] bool HasDirectChild(const CBaseObject* object) const noexcept;
    [[nodiscard]] CBaseObject* FindChildObject(std::int32_t type,
                                               std::int32_t id,
                                               const CGUID& guid) noexcept;
    [[nodiscard]] CBaseObject* RecursiveFindObject(std::int32_t type,
                                                   std::int32_t id) noexcept;
    [[nodiscard]] CBaseObject* RecursiveFindObject(std::int32_t type,
                                                   std::string_view name) noexcept;

    CBaseObject& AddObject(Child object);
    [[nodiscard]] Child RemoveObject(CBaseObject* object) noexcept;
    bool DeleteChildObject(CBaseObject* object) noexcept;
    bool DeleteChildObject(std::int32_t type, std::int32_t id, const CGUID& guid) noexcept;
    void DeleteAllChildObjects(const CBaseObject* except = nullptr) noexcept;

    void SetType(std::int32_t type) noexcept { m_Type = type; }
    void SetID(std::int32_t id) noexcept { m_Id = id; }
    void SetGUID(const CGUID& guid) noexcept { m_ExId = guid; }
    void SetGraphicsID(std::int32_t graphicsId) noexcept { m_GraphicsId = graphicsId; }
    void SetName(std::string_view name);

    [[nodiscard]] std::int32_t GetType() const noexcept { return m_Type; }
    [[nodiscard]] std::int32_t GetID() const noexcept { return m_Id; }
    [[nodiscard]] std::int32_t GetGraphicsID() const noexcept { return m_GraphicsId; }
    [[nodiscard]] const std::string& GetName() const noexcept { return m_Name; }
    [[nodiscard]] const CGUID& GetGUID() const noexcept { return m_ExId; }
    [[nodiscard]] CBaseObject* GetFather() const noexcept { return m_Father; }
    [[nodiscard]] bool IncludesChildren() const noexcept { return m_IncludeChild; }
    void SetIncludeChildren(bool value) noexcept { m_IncludeChild = value; }

protected:
    std::int32_t m_Type{};
    std::int32_t m_Id{};
    CGUID m_ExId{};
    std::int32_t m_GraphicsId{};
    std::string m_Name;
    bool m_IncludeChild{true};
    CBaseObject* m_Father{};
    std::list<Child> m_Children;
};
