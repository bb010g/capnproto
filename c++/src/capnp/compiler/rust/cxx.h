#pragma once

namespace capnp {
namespace compiler {

typedef int cxx_int;

#ifdef CAPNP_COMPILER_NAMESPACED_MAIN
int main(int argc, char* argv[]);
#endif

}
}
