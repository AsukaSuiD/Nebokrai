#include "organizing.h"

#include <algorithm>
#include <cstring>
#include <type_traits>

namespace {
template <class T>
void Append(std::vector<std::uint8_t>& output, const T value)
{
    static_assert(std::is_trivially_copyable_v<T>);
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), bytes, bytes + sizeof(value));
}

template <std::size_t Size>
std::string_view View(const std::array<char, Size>& value) noexcept
{
    const auto end = std::find(value.begin(), value.end(), '\0');
    return {value.data(), static_cast<std::size_t>(end - value.begin())};
}

template <std::size_t Size>
bool Assign(std::array<char, Size>& destination, const std::string_view value) noexcept
{
    if (value.size() >= Size) return false;
    destination.fill('\0');
    std::copy(value.begin(), value.end(), destination.begin());
    return true;
}

template <std::size_t Size>
bool AppendCString(std::vector<std::uint8_t>& output, const std::array<char, Size>& value)
{
    const auto end = std::find(value.begin(), value.end(), '\0');
    if (end == value.end()) return false;
    output.insert(output.end(), value.begin(), std::next(end));
    return true;
}
}

std::string_view OrganizingMemberInfo::Name() const noexcept { return View(name); }
std::string_view OrganizingMemberInfo::Title() const noexcept { return View(title); }
std::string_view OrganizingMemberInfo::Region() const noexcept { return View(region); }
bool OrganizingMemberInfo::SetName(const std::string_view value) noexcept { return Assign(name, value); }
bool OrganizingMemberInfo::SetTitle(const std::string_view value) noexcept { return Assign(title, value); }
bool OrganizingMemberInfo::SetRegion(const std::string_view value) noexcept { return Assign(region, value); }

bool OrganizingMemberInfo::Serialize(std::vector<std::uint8_t>& output) const
{
    Append(output, id);
    if (!AppendCString(output, name)) return false;
    Append(output, level);
    Append(output, occupation);
    Append(output, jobLevel);
    if (!AppendCString(output, title)) return false;
    for (const auto right : purview) Append(output, static_cast<std::int32_t>(right));
    if (!AppendCString(output, region)) return false;
    Append(output, lastOnlineTime);
    Append(output, static_cast<std::uint8_t>(contribute));
    return true;
}
