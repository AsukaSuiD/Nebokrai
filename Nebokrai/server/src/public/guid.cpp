#include "guid.h"

#include <openssl/rand.h>

#include <charconv>
#include <cstdio>
#include <string_view>

namespace
{
constexpr std::array<std::uint8_t, 16> kActiveGuidBytes{
    0xF5, 0xE7, 0x2C, 0xC0, 0xF4, 0x35, 0x2D, 0x48,
    0xB9, 0x27, 0xBB, 0xC9, 0x89, 0x3A, 0xC6, 0xAD};

bool ParseHexByte(std::string_view text, std::uint8_t& value) noexcept
{
    unsigned int parsed = 0;
    const auto result = std::from_chars(
        text.data(), text.data() + text.size(), parsed, 16);
    if (result.ec != std::errc{} || result.ptr != text.data() + text.size() ||
        parsed > 0xFFU) {
        return false;
    }
    value = static_cast<std::uint8_t>(parsed);
    return true;
}

bool ParseLegacyGuid(std::string_view text,
                     std::array<std::uint8_t, 16>& bytes) noexcept
{
    if (text.size() != 38U || text.front() != '{' || text.back() != '}' ||
        text[9] != '-' || text[14] != '-' || text[19] != '-' || text[24] != '-') {
        return false;
    }

    constexpr std::array<std::size_t, 16> kByteOffsets{
        7, 5, 3, 1,
        12, 10,
        17, 15,
        20, 22,
        25, 27, 29, 31, 33, 35};
    for (std::size_t index = 0; index < bytes.size(); ++index) {
        if (!ParseHexByte(text.substr(kByteOffsets[index], 2U), bytes[index])) {
            return false;
        }
    }
    return true;
}

std::uint32_t ReadLe32(std::span<const std::uint8_t, 16> bytes,
                       std::size_t offset) noexcept
{
    return static_cast<std::uint32_t>(bytes[offset]) |
           (static_cast<std::uint32_t>(bytes[offset + 1U]) << 8U) |
           (static_cast<std::uint32_t>(bytes[offset + 2U]) << 16U) |
           (static_cast<std::uint32_t>(bytes[offset + 3U]) << 24U);
}
}

const CGUID CGUID::GUID_INVALID{};
const CGUID CGUID::GUID_IN_ACTIVE = CGUID::FromBytes(kActiveGuidBytes);

CGUID::CGUID(const char* text) noexcept
{
    if (text == nullptr) {
        return;
    }
    std::array<std::uint8_t, 16> parsed{};
    if (ParseLegacyGuid(text, parsed)) {
        m_Bytes = parsed;
    }
}

void CGUID::Initialize() noexcept
{
}

void CGUID::Uninitialize() noexcept
{
}

bool CGUID::CreateGUID(CGUID& guid) noexcept
{
    std::array<std::uint8_t, 16> bytes{};
    if (RAND_bytes(bytes.data(), static_cast<int>(bytes.size())) != 1) {
        guid = GUID_INVALID;
        return false;
    }

    // В смешанном порядке Microsoft старший полубайт версии находится
    // в байте 7, а вариант RFC остаётся в байте 8.
    bytes[7] = static_cast<std::uint8_t>((bytes[7] & 0x0FU) | 0x40U);
    bytes[8] = static_cast<std::uint8_t>((bytes[8] & 0x3FU) | 0x80U);
    guid = FromBytes(bytes);
    return true;
}

bool CGUID::tostring(char* destination, std::size_t capacity) const noexcept
{
    if (destination == nullptr || capacity == 0U) {
        return false;
    }

    const std::uint32_t data1 = ReadLe32(m_Bytes, 0U);
    const std::uint16_t data2 = static_cast<std::uint16_t>(
        m_Bytes[4] | (static_cast<std::uint16_t>(m_Bytes[5]) << 8U));
    const std::uint16_t data3 = static_cast<std::uint16_t>(
        m_Bytes[6] | (static_cast<std::uint16_t>(m_Bytes[7]) << 8U));
    const int written = std::snprintf(
        destination,
        capacity,
        "{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}",
        data1,
        static_cast<unsigned int>(data2),
        static_cast<unsigned int>(data3),
        static_cast<unsigned int>(m_Bytes[8]),
        static_cast<unsigned int>(m_Bytes[9]),
        static_cast<unsigned int>(m_Bytes[10]),
        static_cast<unsigned int>(m_Bytes[11]),
        static_cast<unsigned int>(m_Bytes[12]),
        static_cast<unsigned int>(m_Bytes[13]),
        static_cast<unsigned int>(m_Bytes[14]),
        static_cast<unsigned int>(m_Bytes[15]));
    if (written < 0 || static_cast<std::size_t>(written) >= capacity) {
        destination[0] = '\0';
        return false;
    }
    return true;
}

bool CGUID::IsInvalided() const noexcept
{
    return *this == GUID_INVALID;
}

std::size_t hash_guid_compare::operator()(const CGUID& value) const noexcept
{
    const auto bytes = value.Bytes();
    return static_cast<std::size_t>(ReadLe32(bytes, 0U)) +
           static_cast<std::size_t>(ReadLe32(bytes, 4U)) +
           static_cast<std::size_t>(ReadLe32(bytes, 8U)) +
           static_cast<std::size_t>(ReadLe32(bytes, 12U));
}
