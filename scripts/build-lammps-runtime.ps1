param(
    [Parameter(Mandatory=$true)][string]$Source,
    [string]$Build = 'artifacts/lammps-portable'
)
$ErrorActionPreference = 'Stop'
# Source is the unchanged stable_29Aug2024_update3 tree. Requires VS C++ x64 and CMake.
cmake -S "$Source/cmake" -B $Build -A x64 -D BUILD_SHARED_LIBS=ON -D BUILD_MPI=OFF -D BUILD_OMP=OFF -D CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded
if ($LASTEXITCODE -ne 0) { throw 'LAMMPS configure failed' }
cmake --build $Build --config Release --target lammps --parallel 4
if ($LASTEXITCODE -ne 0) { throw 'LAMMPS build failed' }
Copy-Item -LiteralPath "$Build/Release/liblammps.dll" -Destination liblammps.dll -Force
