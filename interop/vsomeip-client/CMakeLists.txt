cmake_minimum_required(VERSION 3.10)
project(vsomeip_interop_client CXX)

set(CMAKE_CXX_STANDARD 17)

find_package(vsomeip3 REQUIRED)

add_executable(vsomeip_client vsomeip_client.cpp)
target_link_libraries(vsomeip_client vsomeip3)
