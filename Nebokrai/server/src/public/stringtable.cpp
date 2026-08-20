#include "stringtable.h"

#include <cctype>
#include <fstream>
#include <limits>

namespace
{
void SkipSpaceAndComments(std::string_view text, std::size_t& offset)
{
    while (offset < text.size()) {
        if (text[offset] == ';') {
            const auto end = text.find('\n', offset);
            offset = end == std::string_view::npos ? text.size() : end + 1;
            continue;
        }
        if (std::isspace(static_cast<unsigned char>(text[offset])) == 0) break;
        ++offset;
    }
}

template<class T>
void Append(std::vector<std::uint8_t>& output, const T value)
{
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), bytes, bytes + sizeof(value));
}

void AppendCString(std::vector<std::uint8_t>& output, const std::string_view value)
{
    output.insert(output.end(), value.begin(), value.end());
    output.push_back(0);
}
}

bool StringTable::Load(const std::filesystem::path& path, std::string& error)
{
    std::ifstream input(path, std::ios::binary);
    if (!input) {
        error = "не найден языковой пакет " + path.string();
        return false;
    }
    const std::string text((std::istreambuf_iterator<char>(input)), {});
    return Load(std::string_view{text}, error);
}

bool StringTable::Load(const std::string_view text, std::string& error)
{
    std::size_t offset{};
    while (offset < text.size()) {
        SkipSpaceAndComments(text, offset);
        if (offset == text.size()) break;

        const std::size_t idBegin = offset;
        while (offset < text.size() && text[offset] != '"' &&
               std::isspace(static_cast<unsigned char>(text[offset])) == 0) {
            ++offset;
        }
        std::string id(text.substr(idBegin, offset - idBegin));
        const std::size_t quote = text.find('"', offset);
        if (id.empty() || quote == std::string_view::npos) {
            error = "повреждена запись языкового пакета";
            return false;
        }
        const std::size_t valueEnd = text.find('"', quote + 1);
        if (valueEnd == std::string_view::npos) {
            error = "не закрыто значение языковой строки " + id;
            return false;
        }
        m_Entries.insert_or_assign(std::move(id),
                                  std::string(text.substr(quote + 1, valueEnd - quote - 1)));
        offset = valueEnd + 1;
    }
    return true;
}

const std::string* StringTable::Find(const std::string_view id) const noexcept
{
    const auto found = m_Entries.find(id);
    return found == m_Entries.end() ? nullptr : &found->second;
}

std::string StringTable::Resolve(const std::string_view id) const
{
    if (const std::string* value = Find(id)) return *value;
    return std::string(id);
}

bool StringTable::Serialize(std::vector<std::uint8_t>& output) const
{
    if (m_Entries.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max())) {
        return false;
    }
    Append(output, static_cast<std::int32_t>(m_Entries.size()));
    for (const auto& [id, value] : m_Entries) {
        AppendCString(output, id);
        AppendCString(output, value);
    }
    return true;
}
