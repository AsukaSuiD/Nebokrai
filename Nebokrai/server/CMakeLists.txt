cmake_minimum_required(VERSION 3.24)

project(NebokraiServer LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(asio CONFIG REQUIRED)
find_package(CURL CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
find_package(ZLIB REQUIRED)
find_package(Freetype REQUIRED)
find_package(spdlog CONFIG REQUIRED)
find_package(Iconv REQUIRED)
find_package(ODBC REQUIRED)

if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    # Статический unixODBC из vcpkg использует libltdl, но его CMake-цель
    # не передаёт эту зависимость конечному исполняемому файлу.
    find_library(LIBLTDL_LIBRARY NAMES ltdl REQUIRED)
endif()

file(GLOB_RECURSE NEBOKRAI_SERVER_CANDIDATES CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp")
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
    ${NEBOKRAI_SERVER_CANDIDATES})

# В дереве заранее присутствуют пустые owner-файлы. Они обозначают структуру,
# но не должны создавать сотни бессодержательных единиц трансляции.
set(NEBOKRAI_SERVER_SOURCES)
foreach(source IN LISTS NEBOKRAI_SERVER_CANDIDATES)
    file(SIZE "${source}" source_size)
    if(source_size GREATER 0)
        list(APPEND NEBOKRAI_SERVER_SOURCES "${source}")
    endif()
endforeach()

add_library(nebokrai_server STATIC ${NEBOKRAI_SERVER_SOURCES})
add_library(Nebokrai::Server ALIAS nebokrai_server)

target_include_directories(nebokrai_server
    PUBLIC
        "$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/src>")

target_compile_definitions(nebokrai_server
    PUBLIC
        ASIO_STANDALONE
    PRIVATE
        $<$<PLATFORM_ID:Windows>:NOMINMAX>
        $<$<PLATFORM_ID:Windows>:WIN32_LEAN_AND_MEAN>
        $<$<PLATFORM_ID:Windows>:_WIN32_WINNT=0x0601>)

target_link_libraries(nebokrai_server
    PUBLIC
        asio::asio
    PRIVATE
        CURL::libcurl
        OpenSSL::Crypto
        ZLIB::ZLIB
        Freetype::Freetype
        spdlog::spdlog
        Iconv::Iconv
        ODBC::ODBC)

if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    target_link_libraries(nebokrai_server PRIVATE "${LIBLTDL_LIBRARY}")
endif()

add_executable(nebokrai_authserver
    apps/authserver/main.cpp)

target_link_libraries(nebokrai_authserver
    PRIVATE
        Nebokrai::Server
        spdlog::spdlog)

add_executable(nebokrai_loginserver
    apps/loginserver/main.cpp)

target_link_libraries(nebokrai_loginserver
    PRIVATE
        Nebokrai::Server
        spdlog::spdlog)

add_executable(nebokrai_billingserver
    apps/billingserver/main.cpp)

target_link_libraries(nebokrai_billingserver
    PRIVATE
        Nebokrai::Server
        spdlog::spdlog)
