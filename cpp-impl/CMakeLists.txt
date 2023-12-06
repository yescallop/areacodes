cmake_minimum_required(VERSION 3.27)
project(areacodes VERSION 0.1.0)

file(GLOB srcs src/*.cpp include/*.hpp)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED True)

include(FetchContent)

FetchContent_Declare(glaze URL https://github.com/stephenberry/glaze/archive/refs/tags/v1.9.0.tar.gz)
FetchContent_MakeAvailable(glaze)

add_compile_options("$<$<CXX_COMPILER_ID:MSVC>:/utf-8>")
add_executable(${PROJECT_NAME} ${srcs})
target_include_directories(${PROJECT_NAME} PRIVATE include)
target_link_libraries(${PROJECT_NAME} PRIVATE glaze::glaze)
