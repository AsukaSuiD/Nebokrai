#pragma once

#include <filesystem>
#include <optional>
#include <string_view>
#include <system_error>

/*
 * Исходный владелец: public/tools.cpp / tools.h.
 * Исходный путь в PDB:
 * d:\\complite_version\\fengyun_russia\\trunk\\public\\tools.cpp
 *
 * Материализован доказанный фрагмент глобальных вспомогательных функций:
 * GetFileLength 0x00420710 и IniDecoder 0x00420A90.
 * ResolveLegacyFileAsciiCase — тонкая граница Linux поверх std::filesystem:
 * она сохраняет нечувствительный к ASCII-регистру поиск имени файла Windows,
 * но не меняет содержимое, формат или порядок чтения конфигурации Miracle.
 */

[[nodiscard]] int GetFileLength(char* name);
void IniDecoder(char* input, char* output, int length) noexcept;

[[nodiscard]] std::optional<std::filesystem::path> ResolveLegacyFileAsciiCase(
    const std::filesystem::path& directory,
    std::string_view requestedName,
    std::error_code& error);
