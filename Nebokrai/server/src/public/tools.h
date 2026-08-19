#pragma once

/*
 * Owner: public/tools.cpp / tools.h.
 * Original PDB source path:
 * d:\\complite_version\\fengyun_russia\\trunk\\public\\tools.cpp
 *
 * Материализован только доказанный setup-slice глобальных helpers:
 * GetFileLength 0x00420710 и IniDecoder 0x00420A90.
 */

[[nodiscard]] int GetFileLength(char* name);
void IniDecoder(char* input, char* output, int length) noexcept;
