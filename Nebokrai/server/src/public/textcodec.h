#pragma once

#include <optional>
#include <span>
#include <string>
#include <string_view>

/*
 * Технический слой `Nebokrai` без отдельного исходного владельца. Он централизует
 * системный iconv и не определяет игровые кодировки сам: конкретный вызывающий
 * владелец по-прежнему выбирает исходную и целевую кодировку по подтверждённому
 * формату данных.
 */
namespace Nebokrai
{
struct TextConversionResult
{
    std::optional<std::string> value;
    std::string error;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return value.has_value();
    }
};

[[nodiscard]] TextConversionResult ConvertTextEncoding(
    std::span<const char> source,
    std::string_view sourceEncoding,
    std::string_view destinationEncoding);

[[nodiscard]] TextConversionResult Windows1251ToUtf8(std::string_view source);
}
