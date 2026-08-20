#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <optional>
#include <span>
#include <string>
#include <vector>

/*
 * Исходный владелец: loginserver/applogin/validcode.cpp / validcode.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Исходные пути PDB:
 * Путь владельца в PDB: d:\\complite_version\\fengyun_russia\\trunk\\server\\loginserver\\applogin\\validcode.cpp
 * и validcode.h. RVA: BlendQuadColor 0x00023E30, DrawBitmap 0x00023E80,
 * InitBitmap 0x00023F60, Bresenham 0x00023FD0, GenerateValidCodeString
 * 0x000246C0, CodeToBitmap 0x00024770, AddNoise 0x00024B20,
 * CValidCodeSetup::LoadSetup 0x00024E20, CValidCode ctor 0x000251A0.
 *
 * Универсальная тяжёлая часть не переписывается: FreeType отвечает за TTF и
 * raster glyph, iconv — за GBK -> Unicode, std::filesystem — за файлы/пути.
 * Наш код сохраняет только семантику Miracle поверх библиотек: четыре
 * двухбайтовых GBK-символа, исходное modulo-RNG распределение, размеры/
 * повороты/координаты glyph, отсутствие alpha-blend, собственные линии/шум и
 * точный BMP wire.
 *
 * Bitmap всегда ровно 0x70B6 bytes: 14-byte file header + 40-byte info header
 * + 200*48*3 BGR. VERIFIED_DISASSEMBLY: bfSize остаётся ошибочным 0xF6, а
 * biSizeImage — 0xC0; исправлять их нельзя. Glyph intensity не смешивается с
 * цветом: любой ненулевой байт FreeType bitmap означает полный выбранный цвет.
 * Исходный indexing использует bitmap.width, а не pitch — это также сохранено.
 *
 * ValidCode.ini читается исходной whitespace-последовательностью: charset,
 * три signed параметра шума и список font-файлов. Linux case-sensitive пути
 * разрешаются ASCII-case-insensitive внутри runtime-каталога, как уже было
 * принято поздней реконструкцией для Windows-совместимых assets.
 */
namespace Login
{
inline constexpr std::size_t kValidCodeBitmapLength = 0x70B6U;
inline constexpr std::size_t kValidCodeLength = 8U;

enum class ValidCodeErrorKind
{
    Io,
    InvalidSetup,
    InvalidCharset,
    Random,
    Encoding,
    FreeType,
};

struct ValidCodeError
{
    ValidCodeErrorKind kind{};
    std::string detail;
};

class CValidCode
{
public:
    CValidCode() = default;

    [[nodiscard]] static std::optional<ValidCodeError>
    Generate(const std::filesystem::path& runtimeDirectory, CValidCode& output);

    [[nodiscard]] std::span<const std::uint8_t> ValidCode() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t, kValidCodeBitmapLength>
    Bitmap() const noexcept;

private:
    std::array<std::uint8_t, kValidCodeLength> m_ValidCode{};
    std::array<std::uint8_t, kValidCodeBitmapLength> m_Bitmap{};
};
}
