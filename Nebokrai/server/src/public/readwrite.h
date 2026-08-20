#pragma once

#include <istream>

/*
 * Исходный владелец: public/readwrite.cpp / readwrite.h.
 * Исходный путь в PDB:
 * d:\\complite_version\\fengyun_russia\\trunk\\public\\readwrite.cpp
 *
 * Материализованный прямой помощник: ReadTo 0x00420AC0.
 */
[[nodiscard]] bool ReadTo(std::istream& stream, const char* name);
