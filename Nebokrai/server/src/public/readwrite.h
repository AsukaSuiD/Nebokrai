#pragma once

#include <istream>
#include <string_view>

/*
 * Исходный владелец: public/readwrite.cpp / readwrite.h.
 * Исходный путь в PDB:
 * d:\\complite_version\\fengyun_russia\\trunk\\public\\readwrite.cpp
 *
 * Материализованный прямой помощник: ReadTo 0x00420AC0.
 */
[[nodiscard]] bool ReadTo(std::istream& stream, const char* name);

// BillingServer использовал тот же owner для отдельного журнала отклонённых
// покупок. Текст передаётся как уже сформированная операторская запись.
void PutStringToFile(std::string_view filePrefix, std::string_view text);
