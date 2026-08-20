#pragma once

#include <cstdint>
#include <filesystem>
#include <span>

/*
 * Исходный владелец: public/crc32static.cpp
 *
 * Исходный путь владельца в PDB: public/crc32static.cpp; DataCrc32 подтверждён во всех
 * шести серверных EXE/PDB (Auth RVA 0x15F80, Billing 0x13530, Login 0x7F050,
 * Misc 0x5730, Game 0x7B0A0, World 0xA43A0). Варианты подтверждают один и тот же
 * reflected IEEE CRC-32: начальный регистр 0xFFFFFFFF, стандартный reflected
 * polynomial и финальная инверсия. Это стандартный алгоритм, а не особенность
 * Nebokrai, поэтому исходная статическая таблица и ручной цикл заменены zlib.
 *
 * ServerUpdate использовал тот же CRC для файлов через Windows file mapping.
 * Потоковое чтение сохраняет checksum, не перенося Windows mapping plumbing.
 */

[[nodiscard]] std::uint32_t DataCrc32(std::span<const std::uint8_t> data) noexcept;

enum class FileCrc32Status
{
    Ok,
    OpenFailed,
    ReadFailed,
};

struct FileCrc32Result
{
    FileCrc32Status status{FileCrc32Status::OpenFailed};
    std::uint32_t crc32{};

    [[nodiscard]] constexpr explicit operator bool() const noexcept
    {
        return status == FileCrc32Status::Ok;
    }
};

[[nodiscard]] FileCrc32Result FileCrc32(const std::filesystem::path& path);
