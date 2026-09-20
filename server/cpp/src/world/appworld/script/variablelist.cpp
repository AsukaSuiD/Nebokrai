#include "variablelist.h"

#include <algorithm>
#include <charconv>
#include <limits>
#include <sstream>

namespace {
template <class T> void Append(std::vector<std::uint8_t>& output, const T value)
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

bool CVariableList::AddScalar(std::string name, const std::int32_t initial)
{
    if (Find(name)) return false;
    m_Variables.push_back({std::move(name), {initial}, {initial}, {}, {}, false});
    return true;
}

bool CVariableList::AddArray(std::string name, std::vector<std::int32_t> initial)
{
    if (Find(name) || initial.empty()) return false;
    auto saved = initial;
    m_Variables.push_back({std::move(name), std::move(initial), std::move(saved), {}, {}, false});
    return true;
}

bool CVariableList::AddString(std::string name, std::string initial)
{
    if (Find(name)) return false;
    auto saved = initial;
    m_Variables.push_back({std::move(name), {}, {}, std::move(initial), std::move(saved), true});
    return true;
}

bool CVariableList::Restore(std::string name, std::string saved, std::string current)
{
    if (Find(name)) return false;
    const auto unquote = [](std::string value) {
        if (value.size() >= 2 && value.front() == '"' && value.back() == '"')
            return value.substr(1, value.size() - 2);
        return value;
    };
    if ((!saved.empty() && saved.front() == '"') || (!current.empty() && current.front() == '"')) {
        m_Variables.push_back({std::move(name), {}, {}, unquote(std::move(current)),
                               unquote(std::move(saved)), true});
        return true;
    }

    const auto parse = [](const std::string& text, std::vector<std::int32_t>& output) {
        std::istringstream input(text);
        std::int64_t value{};
        while (input >> value) {
            if (value < std::numeric_limits<std::int32_t>::min() ||
                value > std::numeric_limits<std::int32_t>::max()) return false;
            output.push_back(static_cast<std::int32_t>(value));
            while (input.peek() == ',' || input.peek() == ';') input.get();
        }
        return input.eof() && !output.empty();
    };
    std::vector<std::int32_t> savedValues, currentValues;
    if (!parse(saved, savedValues) || !parse(current, currentValues) ||
        savedValues.size() != currentValues.size()) return false;
    m_Variables.push_back({std::move(name), std::move(currentValues), std::move(savedValues),
                           {}, {}, false});
    return true;
}

bool CVariableList::Set(const std::string_view name, const std::size_t index, const std::int32_t value) noexcept
{
    auto* variable = FindMutable(name);
    if (!variable || variable->isString || index >= variable->values.size()) return false;
    variable->values[index] = value;
    return true;
}

bool CVariableList::Set(const std::string_view name, std::string value)
{
    auto* variable = FindMutable(name);
    if (!variable || !variable->isString) return false;
    variable->stringValue = std::move(value);
    return true;
}

std::optional<std::int32_t> CVariableList::Get(const std::string_view name, const std::size_t index) const noexcept
{
    const auto* variable = Find(name);
    if (!variable || variable->isString || index >= variable->values.size()) return std::nullopt;
    return variable->values[index];
}

const CVariableList::Variable* CVariableList::Find(const std::string_view name) const noexcept
{
    const auto found = std::ranges::find(m_Variables, name, &Variable::name);
    return found == m_Variables.end() ? nullptr : &*found;
}

CVariableList::Variable* CVariableList::FindMutable(const std::string_view name) noexcept
{
    const auto found = std::ranges::find(m_Variables, name, &Variable::name);
    return found == m_Variables.end() ? nullptr : &*found;
}

std::vector<VariableSaveRow> CVariableList::SaveRows() const
{
    std::vector<VariableSaveRow> rows;
    rows.reserve(m_Variables.size());
    for (const auto& variable : m_Variables) {
        if (variable.values.size() > 1) rows.push_back({variable.name, {}, {}});
        else if (variable.isString) rows.push_back({variable.name, '"' + variable.savedStringValue + '"', '"' + variable.stringValue + '"'});
        else rows.push_back({variable.name, std::to_string(variable.savedValues.front()), std::to_string(variable.values.front())});
    }
    return rows;
}

bool CVariableList::Serialize(std::vector<std::uint8_t>& output) const
{
    if (m_Variables.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max())) return false;
    Append(output, static_cast<std::int32_t>(m_Variables.size()));
    for (const auto& variable : m_Variables) {
        AppendCString(output, variable.name);
        Append(output, static_cast<std::int32_t>(variable.values.size()));
        if (variable.isString) AppendCString(output, variable.stringValue);
        else for (const auto value : variable.values) Append(output, value);
    }
    return true;
}
