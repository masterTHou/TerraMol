/** Validate local observation files before handing them to either renderer. No dynamics here. */
export function validateTrajectory(data) {
  const fail=()=>{throw new Error('无效的 TerraMol 轨迹：请检查格式、坐标和索引');};
  if(data?.trajectory_schema!=='terramol-render-trajectory-1'||!Array.isArray(data.frames)||!data.frames.length||data.frames.length>1000)fail();
  const vec=v=>Array.isArray(v)&&v.length===3&&v.every(x=>Number.isFinite(x)&&Math.abs(x)<=1e12);
  const version=v=>typeof v==='string'&&/^\d{1,20}$/.test(v);
  for(const f of data.frames){
    if(!f||!Array.isArray(f.atoms)||f.atoms.length>12000||!Array.isArray(f.bonds)||f.bonds.length>12000||!Array.isArray(f.molecules)||f.molecules.length>1000||!Array.isArray(f.elements)||f.elements.length>118||!Number.isFinite(f.time_fs)||!Number.isFinite(f.box_size)||f.box_size<=0||!version(f.topology_version)||!version(f.steps)||!Number.isFinite(f.timestep_fs)||f.timestep_fs<=0||!['native','lj'].includes(f.model))fail();
    if(!['kinetic_kcal_mol','measured_temperature','energy_change_kcal_mol'].every(k=>Number.isFinite(f[k]))||!['potential_kcal_mol','total_kcal_mol'].every(k=>f[k]===null||Number.isFinite(f[k]))||!Number.isSafeInteger(f.degrees_of_freedom)||f.degrees_of_freedom<0)fail();
    if(f.elements.some(e=>!e||!Number.isInteger(e.number)||!vec(e.cpk)||e.cpk.some(x=>x<0||x>1)||!Number.isFinite(e.vdw)||e.vdw<=0||e.vdw>100))fail();
    const elements=new Set(f.elements.map(e=>e.number));
    if(elements.size!==f.elements.length)fail();
    if(f.molecules.some(m=>!m||!version(m.id)||typeof m.name!=='string'||m.name.length>1000||!vec(m.center)||!vec(m.velocity)||!vec(m.angular_velocity)||!Number.isFinite(m.mass_amu)||m.mass_amu<=0))fail();
    const molecules=new Set(f.molecules.map(m=>m.id));if(molecules.size!==f.molecules.length)fail();
    if(f.atoms.some(a=>!a||!vec(a.position)||!vec(a.velocity)||!elements.has(a.element)||!molecules.has(a.molecule_id)))fail();
    if(f.bonds.some(b=>!b||!Number.isInteger(b.a)||!Number.isInteger(b.b)||b.a<0||b.b<0||b.a>=f.atoms.length||b.b>=f.atoms.length||b.a===b.b))fail();
  }
  return data.frames;
}
