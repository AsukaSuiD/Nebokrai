#include "tools.h"

#include <cstdint>
#include <cstdio>
#include <limits>

int GetFileLength(char* name)
{
    // VERIFIED_ASSEMBLY 0x00420710: null -> 0; open failure -> -1;
    // otherwise length of the opened file and close.
    if (name == nullptr) {
        return 0;
    }

    std::FILE* file = std::fopen(name, "rb");
    if (file == nullptr) {
        return -1;
    }
    if (std::fseek(file, 0, SEEK_END) != 0) {
        std::fclose(file);
        return -1;
    }
    const long length = std::ftell(file);
    std::fclose(file);
    if (length < 0 ||
        static_cast<unsigned long>(length) >
            static_cast<unsigned long>(std::numeric_limits<int>::max())) {
        // Old 32-bit _filelength cannot represent a larger setup.dat either.
        return -1;
    }
    return static_cast<int>(length);
}

void IniDecoder(char* input, char* output, int length) noexcept
{
    if (input == nullptr || output == nullptr || length <= 0) {
        // Direct caller always supplies valid buffers; this only replaces legacy
        // null/OOB UB at the technical boundary.
        return;
    }

    // VERIFIED_ASSEMBLY 0x00420A90:
    //   dl = input[i]; sub dl,0x0C; not dl; output[i] = dl.
    for (int index = 0; index < length; ++index) {
        const std::uint8_t source =
            static_cast<std::uint8_t>(static_cast<unsigned char>(input[index]));
        const std::uint8_t subtracted =
            static_cast<std::uint8_t>(source - std::uint8_t{0x0C});
        output[index] = static_cast<char>(
            static_cast<std::uint8_t>(~subtracted));
    }
}
