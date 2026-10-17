import {validateTrajectory,TrajectoryRecorder} from '../viewer/trajectory.js';
import assert from 'node:assert/strict';
const frame={model:'native',topology_version:'1',steps:'10',timestep_fs:1,time_fs:10,box_size:40,
  kinetic_kcal_mol:1,potential_kcal_mol:null,total_kcal_mol:null,measured_temperature:300,energy_change_kcal_mol:0,degrees_of_freedom:6,
  atoms:[{element:1,molecule_id:'1',position:[20,20,20],velocity:[0,0,0]},{element:1,molecule_id:'1',position:[21,20,20],velocity:[0,0,0]}],
  bonds:[{a:0,b:1}],molecules:[{id:'1',name:'test',center:[20.5,20,20],velocity:[0,0,0],angular_velocity:[0,0,0],mass_amu:2.016}],
  elements:[{number:1,cpk:[1,1,1],vdw:1.2}]};
const fixture=()=>({trajectory_schema:'terramol-render-trajectory-1',frames:[structuredClone(frame)]});
assert.equal(validateTrajectory(fixture()).length,1);
for(const mutate of [
 d=>d.frames[0].bonds[0].b=2,
 d=>d.frames[0].atoms[0].position[0]=Infinity,
 d=>d.frames[0].atoms[0].molecule_id='2',
 d=>d.frames[0].atoms[0].element=999,
 d=>d.frames[0].molecules[0].angular_velocity=[0,NaN,0],
 d=>d.frames[0].elements[0].vdw=-1,
 d=>d.frames[0].steps='not an integer',
 d=>d.frames[0].total_kcal_mol='0',
 d=>d.frames=Array(1001).fill(frame),
 d=>d.trajectory_schema='unknown'
]){const d=fixture();mutate(d);assert.throws(()=>validateTrajectory(d));}
console.log('Trajectory file validation: valid observations accepted, malformed references/coordinates/limits rejected');

const bounded=new TrajectoryRecorder(2,100000);assert(bounded.append(frame));
frame.atoms[0].position[0]=30;assert.equal(bounded.frames[0].atoms[0].position[0],20);
assert(bounded.append(frame));assert.equal(bounded.append(frame),false);assert.equal(bounded.frames.length,2);
const budget=new TrajectoryRecorder(1000,10);assert.equal(budget.append(frame),false);assert.equal(budget.frames.length,0);
const exact=new TrajectoryRecorder(1000,new TextEncoder().encode(JSON.stringify(frame)).length);
assert(exact.append(frame));assert.equal(exact.append(frame),false);
console.log('Trajectory recording: copy isolation, frame cap and byte budget enforced');
