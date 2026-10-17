#ifndef MOL_ENGINE_CORE_H
#define MOL_ENGINE_CORE_H
#include <stdint.h>
#ifdef _WIN32
#define ME_CALL __cdecl
#else
#define ME_CALL
#endif
#ifdef __cplusplus
extern "C" {
#endif
/* All calls return 0 success, 1 invalid, 2 not found, 3 capacity, 4 backend,
 * 5 panic, 6 unsupported. No pointer is retained. Errors are thread-local.
 * Handles are opaque nonzero tokens, not addresses. Release exactly once.
 * Buffers and outputs must be valid and non-overlapping; dangling/arbitrary
 * addresses cannot be validated by a C ABI. Never mutate buffers during calls. */
typedef uint64_t me_handle;
typedef struct {uint32_t abi_version,header_bytes,atom_count,bond_count;uint64_t state_version,topology_version;uint32_t atom_stride,bond_stride;} me_snapshot_header;
typedef struct {float position[3],velocity[3];uint32_t element,reserved;uint64_t molecule_id;} me_render_atom;
typedef struct {uint32_t a,b,order,reserved;} me_render_bond;
typedef struct {uint32_t number;float cpk[3];double vdw,mass;} me_element_info;
typedef struct {double center[3],velocity[3],mass;} me_molecule_data;
int32_t ME_CALL me_molecule_info(me_handle h,uint64_t id,me_molecule_data *out);
int32_t ME_CALL me_abi_version(uint32_t *out);
int32_t ME_CALL me_create(double box_size,double dt_fs,me_handle *out);
/* Always exported; returns unsupported if built without lammps.
 * Path is UTF-8 bytes without trailing NUL. LJ accepts monatomic single species. */
int32_t ME_CALL me_create_lj(const uint8_t *path,uint32_t length,double box_size,double dt_fs,double epsilon_kcal_mol,double sigma_angstrom,double cutoff_angstrom,me_handle *out);
int32_t ME_CALL me_spawn_particle(me_handle h,uint32_t atomic_number,double x,double y,double z,uint64_t *id);
int32_t ME_CALL me_destroy(me_handle h);
int32_t ME_CALL me_step(me_handle h,uint32_t steps);
int32_t ME_CALL me_reset_template(me_handle h,uint32_t kind,uint32_t count,uint64_t seed,double kelvin);
int32_t ME_CALL me_spawn(me_handle h,uint32_t kind,double x,double y,double z,uint64_t *id);
int32_t ME_CALL me_batch_spawn(me_handle h,uint32_t kind,uint32_t count,uint64_t seed,uint32_t retries);
int32_t ME_CALL me_remove(me_handle h,uint64_t id);
int32_t ME_CALL me_clear(me_handle h);
int32_t ME_CALL me_set_velocity(me_handle h,uint64_t id,double x,double y,double z);
typedef struct {double kinetic,potential,total,temperature;uint32_t degrees_of_freedom,has_potential;} me_observables_data;
/* Energies kcal/mol, temperature K. has_potential=0: potential/total unavailable. */
int32_t ME_CALL me_observables(me_handle h,me_observables_data *out);
/* World angular velocity radians/fs; null-inertia axes are projected out. */
int32_t ME_CALL me_set_angular_velocity(me_handle h,uint64_t id,double x,double y,double z);
int32_t ME_CALL me_thermal(me_handle h,double kelvin,uint64_t seed);
int32_t ME_CALL me_flow(me_handle h,double x,double y,double z);
int32_t ME_CALL me_snapshot_size(me_handle h,uint64_t *bytes);
/* Wire snapshot is LE, 40 byte header, 40 byte atoms, 16 byte bonds. */
int32_t ME_CALL me_snapshot_copy(me_handle h,uint8_t *buffer,uint64_t capacity,uint64_t *required);
int32_t ME_CALL me_element(uint32_t number,me_element_info *out);
int32_t ME_CALL me_last_error(uint8_t *buffer,uint64_t capacity,uint64_t *required);
#ifdef __cplusplus
}
#endif
#endif
