#pragma once

#include <cstdint>
#include <filesystem>
#include <map>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: Public/stringtable.cpp / .h.
 * Формат `ID "значение"`, комментарии `;` и замещение одинаковых ID
 * подтверждены поставочными Language.lag и поздней реконструкцией. Старый
 * CRFile/global CGame здесь не нужен: файловая граница принадлежит caller-у.
 */
class StringTable
{
public:
    [[nodiscard]] bool Load(const std::filesystem::path& path, std::string& error);
    [[nodiscard]] bool Load(std::string_view text, std::string& error);
    void Clear() noexcept { m_Entries.clear(); }

    [[nodiscard]] const std::string* Find(std::string_view id) const noexcept;
    [[nodiscard]] std::string Resolve(std::string_view id) const;
    [[nodiscard]] bool Serialize(std::vector<std::uint8_t>& output) const;
    [[nodiscard]] std::size_t Size() const noexcept { return m_Entries.size(); }

private:
    std::map<std::string, std::string, std::less<>> m_Entries;
};
