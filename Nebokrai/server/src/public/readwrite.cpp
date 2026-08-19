#include "readwrite.h"

#include <string>

bool ReadTo(std::istream& stream, const char* name)
{
    // VERIFIED_ASSEMBLY 0x00420AC0: first token is read unconditionally.
    std::string token;
    stream >> token;

    while (token != name) {
        // Direct owner checks stream.eof() before the literal terminator.
        if (stream.eof() || token == "<end>") {
            return false;
        }
        stream >> token;
    }
    return true;
}
