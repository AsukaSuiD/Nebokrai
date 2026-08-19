#pragma once

#include <istream>

/*
 * Owner: public/readwrite.cpp / readwrite.h.
 * Original PDB source path:
 * d:\\complite_version\\fengyun_russia\\trunk\\public\\readwrite.cpp
 *
 * Materialized direct helper: ReadTo 0x00420AC0.
 */
[[nodiscard]] bool ReadTo(std::istream& stream, const char* name);
