#include "mol_engine_core.h"
#include <stddef.h>
_Static_assert(sizeof(me_snapshot_header)==40,"header layout");
_Static_assert(sizeof(me_render_atom)==40,"atom layout");
_Static_assert(sizeof(me_render_bond)==16,"bond layout");
_Static_assert(sizeof(me_element_info)==32,"element layout");
_Static_assert(sizeof(me_molecule_data)==56,"molecule layout");
_Static_assert(offsetof(me_render_atom,molecule_id)==32,"ID offset");
_Static_assert(offsetof(me_snapshot_header,state_version)==16,"state offset");

_Static_assert(sizeof(me_observables_data)==40,"observables layout");
_Static_assert(offsetof(me_observables_data,has_potential)==36,"potential flag offset");
