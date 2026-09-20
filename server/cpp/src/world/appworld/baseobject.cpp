#include "baseobject.h"

#include <algorithm>
#include <array>
#include <cstring>
#include <ranges>
#include <stdexcept>
#include <type_traits>
#include <utility>

namespace
{
template <class T>
void AppendScalar(std::vector<std::uint8_t>& output, const T& value)
{
    static_assert(std::is_trivially_copyable_v<T>);
    const auto* first = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), first, first + sizeof(T));
}

template <class T>
bool ReadScalar(std::span<const std::uint8_t> input, std::size_t& offset, T& value)
{
    static_assert(std::is_trivially_copyable_v<T>);
    if (offset > input.size() || input.size() - offset < sizeof(T)) {
        return false;
    }
    std::memcpy(&value, input.data() + offset, sizeof(T));
    offset += sizeof(T);
    return true;
}
}

bool CBaseObject::Load()
{
    return true;
}

bool CBaseObject::Save()
{
    return true;
}

bool CBaseObject::AddToByteArray(std::vector<std::uint8_t>& output, bool includeChild) const
{
    (void)includeChild;
    AppendScalar(output, m_Type);
    AppendScalar(output, m_Id);
    AppendScalar(output, m_GraphicsId);
    output.insert(output.end(), m_Name.begin(), m_Name.end());
    output.push_back(0);
    return true;
}

bool CBaseObject::DecordFromByteArray(std::span<const std::uint8_t> input,
                                      std::size_t& offset,
                                      bool includeChild)
{
    (void)includeChild;
    if (!ReadScalar(input, offset, m_Type) || !ReadScalar(input, offset, m_Id) ||
        !ReadScalar(input, offset, m_GraphicsId)) {
        return false;
    }

    const auto begin = input.begin() + static_cast<std::ptrdiff_t>(offset);
    const auto terminator = std::find(begin, input.end(), std::uint8_t{0});
    if (terminator == input.end() || std::distance(begin, terminator) > 255) {
        return false;
    }
    m_Name.assign(reinterpret_cast<const char*>(&*begin),
                  static_cast<std::size_t>(std::distance(begin, terminator)));
    offset += static_cast<std::size_t>(std::distance(begin, terminator)) + 1;
    return true;
}

void CBaseObject::AI()
{
    std::vector<CBaseObject*> snapshot;
    snapshot.reserve(m_Children.size());
    for (const auto& child : m_Children) {
        snapshot.push_back(child.get());
    }
    for (CBaseObject* child : snapshot) {
        if (child != nullptr && child->m_Father == this) {
            child->AI();
        }
    }
}

void CBaseObject::BoardCast(std::int32_t messageId, std::int32_t parameter)
{
    std::vector<CBaseObject*> snapshot;
    snapshot.reserve(m_Children.size());
    for (const auto& child : m_Children) {
        snapshot.push_back(child.get());
    }
    for (CBaseObject* child : snapshot) {
        if (child != nullptr && child->m_Father == this) {
            child->BoardCast(messageId, parameter);
        }
    }
}

void CBaseObject::FindObjectsByType(std::int32_t type,
                                    const ObjectTypeCallback& callback)
{
    if (!callback) {
        return;
    }
    for (const auto& child : m_Children) {
        if (child->GetType() == type) {
            callback(*child);
        } else {
            child->FindObjectsByType(type, callback);
        }
    }
}

bool CBaseObject::HasDirectChild(const CBaseObject* object) const noexcept
{
    return std::ranges::any_of(m_Children,
                               [object](const Child& child) { return child.get() == object; });
}

CBaseObject* CBaseObject::FindChildObject(std::int32_t type,
                                          std::int32_t id,
                                          const CGUID& guid) noexcept
{
    const auto found = std::ranges::find_if(m_Children, [&](const Child& child) {
        return child->GetType() == type && child->GetID() == id &&
               (guid.IsInvalided() || child->GetGUID() == guid);
    });
    return found == m_Children.end() ? nullptr : found->get();
}

CBaseObject* CBaseObject::RecursiveFindObject(std::int32_t type, std::int32_t id) noexcept
{
    if (m_Type == type && m_Id == id) {
        return this;
    }
    for (const auto& child : m_Children) {
        if (CBaseObject* found = child->RecursiveFindObject(type, id)) {
            return found;
        }
    }
    return nullptr;
}

CBaseObject* CBaseObject::RecursiveFindObject(std::int32_t type,
                                              std::string_view name) noexcept
{
    if (m_Type == type && m_Name == name) {
        return this;
    }
    for (const auto& child : m_Children) {
        if (CBaseObject* found = child->RecursiveFindObject(type, name)) {
            return found;
        }
    }
    return nullptr;
}

CBaseObject& CBaseObject::AddObject(Child object)
{
    if (!object) {
        throw std::invalid_argument("Нельзя добавить пустой объект World");
    }
    object->m_Father = this;
    m_Children.push_back(std::move(object));
    return *m_Children.back();
}

CBaseObject::Child CBaseObject::RemoveObject(CBaseObject* object) noexcept
{
    const auto found = std::ranges::find_if(
        m_Children, [object](const Child& child) { return child.get() == object; });
    if (found == m_Children.end()) {
        return {};
    }
    Child detached = std::move(*found);
    m_Children.erase(found);
    detached->m_Father = nullptr;
    return detached;
}

bool CBaseObject::DeleteChildObject(CBaseObject* object) noexcept
{
    return static_cast<bool>(RemoveObject(object));
}

bool CBaseObject::DeleteChildObject(std::int32_t type,
                                    std::int32_t id,
                                    const CGUID& guid) noexcept
{
    return DeleteChildObject(FindChildObject(type, id, guid));
}

void CBaseObject::DeleteAllChildObjects(const CBaseObject* except) noexcept
{
    if (except == nullptr) {
        m_Children.clear();
        return;
    }
    std::erase_if(m_Children, [except](const Child& child) { return child.get() != except; });
}

void CBaseObject::SetName(std::string_view name)
{
    const std::size_t terminator = name.find('\0');
    m_Name.assign(name.substr(0, terminator));
}
