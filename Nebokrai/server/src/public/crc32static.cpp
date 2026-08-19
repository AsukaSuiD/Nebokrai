#include "crc32static.h"

#include <zlib.h>

#include <algorithm>
#include <array>
#include <fstream>
#include <limits>

namespace
{
std::uint32_t UpdateCrc(std::uint32_t current, std::span<const std::uint8_t> data) noexcept
{
    uLong crc = current;
    std::size_t offset = 0;
    while (offset < data.size()) {
        const std::size_t remaining = data.size() - offset;
        const std::size_t chunkSize =
            std::min<std::size_t>(remaining, std::numeric_limits<uInt>::max());
        crc = ::crc32(crc,
                      reinterpret_cast<const Bytef*>(data.data() + offset),
                      static_cast<uInt>(chunkSize));
        offset += chunkSize;
    }
    return static_cast<std::uint32_t>(crc);
}
}

std::uint32_t DataCrc32(std::span<const std::uint8_t> data) noexcept
{
    const auto initial = static_cast<std::uint32_t>(::crc32(0L, Z_NULL, 0));
    return UpdateCrc(initial, data);
}

FileCrc32Result FileCrc32(const std::filesystem::path& path)
{
    std::ifstream input(path, std::ios::binary);
    if (!input) {
        return {FileCrc32Status::OpenFailed, 0};
    }

    std::uint32_t crc = static_cast<std::uint32_t>(::crc32(0L, Z_NULL, 0));
    std::array<std::uint8_t, 64 * 1024> buffer{};
    for (;;) {
        input.read(reinterpret_cast<char*>(buffer.data()),
                   static_cast<std::streamsize>(buffer.size()));
        const std::streamsize count = input.gcount();
        if (count > 0) {
            crc = UpdateCrc(crc,
                            std::span<const std::uint8_t>(
                                buffer.data(), static_cast<std::size_t>(count)));
        }
        if (input.eof()) {
            return {FileCrc32Status::Ok, crc};
        }
        if (!input) {
            return {FileCrc32Status::ReadFailed, 0};
        }
    }
}
