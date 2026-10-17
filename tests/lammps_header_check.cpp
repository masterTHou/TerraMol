// Compile against the exact installed header before loading a new LAMMPS build.
#include <library.h>
#include <type_traits>
static_assert(std::is_same<decltype(&lammps_open_no_mpi), void*(*)(int,char**,void**)>::value,"open ABI");
static_assert(std::is_same<decltype(&lammps_close), void(*)(void*)>::value,"close ABI");
static_assert(std::is_same<decltype(&lammps_command), char*(*)(void*,const char*)>::value,"command ABI");
static_assert(std::is_same<decltype(&lammps_create_atoms), int(*)(void*,int,const int*,const int*,const double*,const double*,const int*,int)>::value,"requires 32-bit tagint");
static_assert(std::is_same<decltype(&lammps_gather_atoms), void(*)(void*,const char*,int,int,void*)>::value,"gather ABI");
static_assert(std::is_same<decltype(&lammps_scatter_atoms), void(*)(void*,const char*,int,int,void*)>::value,"scatter ABI");
static_assert(std::is_same<decltype(&lammps_extract_setting), int(*)(void*,const char*)>::value,"setting ABI");
static_assert(std::is_same<decltype(&lammps_has_error), int(*)(void*)>::value,"error ABI");
static_assert(std::is_same<decltype(&lammps_get_last_error_message), int(*)(void*,char*,int)>::value,"message ABI");
static_assert(std::is_same<decltype(&lammps_get_thermo), double(*)(void*,const char*)>::value,"thermo ABI");
static_assert(std::is_same<decltype(&lammps_config_has_mpi_support), int(*)()>::value,"MPI config ABI");
static_assert(std::is_same<decltype(&lammps_config_has_exceptions), int(*)()>::value,"exception config ABI");
