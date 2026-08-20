#pragma once

#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/script/variablelist.cpp/.h.
 * PDB/Rust подтверждают scalar/string/array представление и DB save-view.
 * Сырые указатели заменены vector, а сохранение выполняется внешним DB-owner
 * по снимку; форматированные значения повторяют старые %d и "строка".
 */
struct VariableSaveRow { std::string name, initialValue, currentValue; };

class CVariableList {
public:
    struct Variable {
        std::string name;
        std::vector<std::int32_t> values;
        std::vector<std::int32_t> savedValues;
        std::string stringValue;
        std::string savedStringValue;
        bool isString{};
    };

    bool AddScalar(std::string name, std::int32_t initialValue);
    bool AddArray(std::string name, std::vector<std::int32_t> initialValues);
    bool AddString(std::string name, std::string initialValue);
    bool Restore(std::string name, std::string savedValue, std::string currentValue);
    bool Set(std::string_view name, std::size_t index, std::int32_t value) noexcept;
    bool Set(std::string_view name, std::string value);
    [[nodiscard]] std::optional<std::int32_t> Get(std::string_view name, std::size_t index = 0) const noexcept;
    [[nodiscard]] const Variable* Find(std::string_view name) const noexcept;
    [[nodiscard]] std::vector<VariableSaveRow> SaveRows() const;
    [[nodiscard]] bool Serialize(std::vector<std::uint8_t>& output) const;

private:
    Variable* FindMutable(std::string_view name) noexcept;
    std::vector<Variable> m_Variables;
};
