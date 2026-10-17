import { validateTrajectory, TrajectoryRecorder } from './trajectory.js';
import { MolecularView } from './renderer.js';
const $=id=>document.getElementById(id), base=new URL('./',location.href);
let state=null,revision=-1n,kind=0,selected=null,closed=false,toastTimer,lastPhysicsError=null,lastConnectionToast=-Infinity;
const names=['水','甲烷','苯','LJ 粒子'];
let recording=false,recorder=new TrajectoryRecorder(),experiment=null,replaying=false,replayFrames=[],lastRecorded=null;
const view=new MolecularView($('scene'),id=>{selected=id;updateSelection(true);},name=>{$('renderer-name').textContent=name;});
function toast(message,error=false){clearTimeout(toastTimer);const box=$('toast');box.textContent=message;box.classList.toggle('error',error);box.hidden=false;toastTimer=setTimeout(()=>{box.hidden=true;},error?6500:2600);}
function moleculeName(m){return m.name.startsWith('Generic LJ')?'通用 LJ 粒子':m.name.startsWith('Water')?'水分子':m.name.startsWith('Methane')?'甲烷分子':m.name.startsWith('Benzene')?'苯分子':m.name;}
function updateSelection(editFields=false){const molecule=state?.molecules.find(m=>m.id===selected);$('selection-empty').hidden=!!molecule;$('selection-info').hidden=!molecule;if(!molecule)return;$('selected-name').textContent=moleculeName(molecule);$('selected-id').textContent='#'+molecule.id;$('selected-mass').textContent=molecule.mass_amu.toFixed(3)+' amu';$('selected-angular').textContent=Math.hypot(...(molecule.angular_velocity||[0,0,0])).toFixed(5)+' rad/fs';$('selected-speed').textContent=Math.hypot(...molecule.velocity).toFixed(5)+' Å/fs';if(editFields)for(const [i,id]of ['wx','wy','wz'].entries())$(id).value=(molecule.angular_velocity||[0,0,0])[i].toFixed(5);if(editFields)for(const [i,id]of ['vx','vy','vz'].entries())$(id).value=molecule.velocity[i].toFixed(5);}
function accept(next){if(closed||replaying)return;const version=BigInt(next.revision);if(version<revision)return;revision=version;state=next;view.setState(next);selected=view.selected;$('molecule-count').textContent=next.molecules.length;$('atom-count').textContent=next.atoms.length;$('time').textContent=(next.time_fs/1000).toFixed(3)+' ps';$('step-count').textContent=Number(next.steps).toLocaleString()+' 个固定物理步';$('play-label').textContent=next.running?'暂停模拟':'继续运行';$('play-icon').textContent=next.running?'Ⅱ':'▶';$('live-badge').textContent=next.running?'LIVE SIMULATION':'SIMULATION PAUSED';$('live-badge').classList.toggle('paused',!next.running);$('empty-scene').hidden=next.atoms.length!==0;$('loading').hidden=true;$('connection').textContent='Rust 核心已连接';$('status-dot').classList.add('online');$('fps').textContent=view.fps+' FPS';updateSelection();if(next.error&&next.error!==lastPhysicsError)toast(next.error,true);lastPhysicsError=next.error;
  if(document.activeElement!==$('speed')){$('speed').value=next.speed;$('speed-value').textContent=next.speed.toFixed(1)+'×';}
  $('model').value=next.model;$('backend-label').textContent=next.model==='lj'?'LAMMPS · LJ 势':'NATIVE · 平动与转动';
  const ljOption=$('model').querySelector('[value="lj"]');ljOption.disabled=!next.lammps_available;ljOption.textContent=next.lammps_available?'LAMMPS · 通用 LJ 粒子':'LAMMPS · 需完整便携包';
  $('simulation-parameters').textContent=`${next.box_size} Å 模拟盒 · 固定步长 ${next.timestep_fs} fs`;
  $('model-note').textContent=next.model==='lj'?'通用单原子 LJ 模型：ε=0.1 kcal/mol，σ=3.4 Å，截断 8.5 Å，势能平移，NVE＋反射墙。示例质量 12.011 amu，不是碳力场。':'刚体保持形状并自由转动。光滑包围球碰撞不产生转动力矩；这不是原子级 MD。';
  document.querySelector('.molecule-options').hidden=next.model==='lj';$('lj-choice').hidden=next.model!=='lj';
  $('measured-temperature').textContent=next.measured_temperature.toFixed(1)+' K';
  const energy=x=>x==null?'不适用':x.toFixed(5);
  $('kinetic').textContent=energy(next.kinetic_kcal_mol);$('potential').textContent=energy(next.potential_kcal_mol);$('total-energy').textContent=energy(next.total_kcal_mol);$('energy-change').textContent=energy(next.energy_change_kcal_mol);$('dof').textContent=next.degrees_of_freedom;
  if(recording&&next.steps!==lastRecorded){
    if(!recorder.append(next)||recorder.frames.length>=1000){recording=false;$('record').textContent='● 开始记录轨迹';toast('已达到记录上限，请导出轨迹。');}
    lastRecorded=next.steps;$('record-status').textContent=`已记录 ${recorder.frames.length} 帧 / ${(recorder.bytes/1048576).toFixed(1)} MiB；上限 1000 帧 / 20 MiB。`;
  }
  const sceneNames=[...new Set(next.molecules.map(moleculeName))];$('scene-title').textContent=sceneNames.length>1?'分子混合场景':sceneNames.length?sceneNames[0]+'的运动':'等待添加分子';
  const present=new Set(next.atoms.map(a=>a.element));const signature=[...present].sort().join(',');if($('legend').dataset.elements!==signature){$('legend').replaceChildren();for(const element of next.elements.filter(e=>present.has(e.number))){const item=document.createElement('span'),dot=document.createElement('i');dot.style.backgroundColor=`rgb(${element.cpk.map(v=>Math.round(v*255)).join(',')})`;item.append(dot,document.createTextNode(element.symbol));$('legend').append(item);}$('legend').dataset.elements=signature;}
}
async function api(path,action){const response=await fetch(new URL(path,base),action?{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(action),signal:AbortSignal.timeout(5000)}:{signal:AbortSignal.timeout(5000)});if(!response.headers.get('content-type')?.includes('application/json'))throw new Error('本机服务暂时繁忙，请稍后重试');const data=await response.json();if(!response.ok)throw new Error(data.error||'操作失败，请重试');return data;}
async function perform(action,success){if(closed)return;if(replaying&&action.action!=='shutdown'){toast('请先返回实时模拟',true);return false;}try{const result=await api('api/action',action);accept(result);if(success)toast(success);return true;}catch(error){toast(error.message,true);return false;}}
function value(id){const input=$(id);if(!input.checkValidity())throw new Error('请检查输入值的范围');const n=Number(input.value);if(input.value.trim()===''||!Number.isFinite(n))throw new Error('请输入有效的数字');return n;}
function guard(fn){return()=>{try{const result=fn();if(result?.catch)result.catch(e=>toast(e.message,true));}catch(error){toast(error.message,true);}};}
for(const button of document.querySelectorAll('[data-kind]'))button.addEventListener('click',()=>{kind=Number(button.dataset.kind);for(const other of document.querySelectorAll('[data-kind]')){other.classList.toggle('active',other===button);other.setAttribute('aria-pressed',String(other===button));}});
$('spawn-one').addEventListener('click',guard(()=>perform({action:'spawn',kind,count:1},'已添加 1 个'+names[kind]+'分子')));
$('spawn-batch').addEventListener('click',guard(()=>perform({action:'spawn',kind,count:50},'已添加 50 个'+names[kind]+'分子')));
const reset=guard(async()=>{if(await perform({action:'reset',kind,seed:42,kelvin:value('temperature')},'已用固定种子重新开始')){view.select(null);$('scene-title').textContent=names[kind]+'分子的运动';}});
$('reset').addEventListener('click',reset);
$('thermal').addEventListener('click',guard(()=>perform({action:'thermal',kelvin:value('temperature')},'已施加平动与转动热速度')));
const playback=()=>{if(state)perform({action:'playback',running:!state.running});};$('play').addEventListener('click',playback);
let speedTimer;$('speed').addEventListener('input',()=>{const speed=Number($('speed').value);$('speed-value').textContent=speed.toFixed(1)+'×';clearTimeout(speedTimer);speedTimer=setTimeout(()=>perform({action:'speed',value:speed}),120);});
$('fit').addEventListener('click',()=>view.fit());
$('velocity').addEventListener('click',guard(()=>{if(selected)return perform({action:'velocity',id:selected,value:['vx','vy','vz'].map(value)},'已设置整个分子的速度');}));
const remove=()=>{if(selected)perform({action:'remove',id:selected},'已删除选中的分子');};$('remove').addEventListener('click',remove);
$('clear').addEventListener('click',()=>perform({action:'clear'},'场景已清空'));
$('exit').addEventListener('click',async()=>{try{await api('api/action',{action:'shutdown'});closed=true;clearTimeout(speedTimer);view.dispose();$('closed').hidden=false;}catch(error){toast(error.message,true);}});
document.addEventListener('keydown',event=>{if(/INPUT|TEXTAREA|SELECT|BUTTON/.test(event.target.tagName))return;if(event.code==='Space'){event.preventDefault();playback();}else if(event.code==='Delete')remove();else if(event.code==='Escape')view.select(null);else if(event.code==='KeyR')reset();});
async function poll(){if(closed)return;const started=performance.now();try{accept(await api('api/state'));}catch(error){if(closed)return;$('connection').textContent='连接已中断';$('status-dot').classList.remove('online');if(!state){$('loading').querySelector('h3').textContent='暂时无法连接模拟';$('loading').querySelector('p').textContent='请重新双击启动入口。';}if(performance.now()-lastConnectionToast>8000){toast('模拟连接中断。若已关闭程序，请重新双击启动入口。',true);lastConnectionToast=performance.now();}}setTimeout(poll,Math.max(30,50-(performance.now()-started)));}
poll();

$('model').addEventListener('change',async()=>{const model=$('model').value;if(await perform({action:'mode',model},'已重新初始化物理模型')){kind=model==='lj'?3:0;for(const button of document.querySelectorAll('[data-kind]')){button.classList.toggle('active',Number(button.dataset.kind)===kind);button.setAttribute('aria-pressed',String(Number(button.dataset.kind)===kind));}$('temperature').value=model==='lj'?100:300;view.select(null);}else if(state)$('model').value=state.model;});
$('angular-velocity').addEventListener('click',guard(()=>{if(selected)return perform({action:'angular_velocity',id:selected,value:['wx','wy','wz'].map(value)},'已设置角速度');}));
$('record').addEventListener('click',guard(async()=>{
  if(replaying){toast('请先返回实时模拟',true);return;}
  if(!recording){experiment=await api('api/experiment');recorder=new TrajectoryRecorder();lastRecorded=null;}
  recording=!recording;$('record').textContent=recording?'■ 停止记录':'● 开始记录轨迹';
}));
$('export').addEventListener('click',guard(async()=>{
  if(!state)throw new Error('请等待场景加载完成');
  const metadata=await api('api/experiment');
  if(recording){recording=false;$('record').textContent='● 开始记录轨迹';}
  const data={...metadata,recording_start:experiment,trajectory_schema:'terramol-render-trajectory-1',frames:recorder.frames.length?recorder.frames:[state]};
  const url=URL.createObjectURL(new Blob([JSON.stringify(data)],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download='TerraMol-experiment.json';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
}));
function showReplay(i){const frame=replayFrames[i];if(!frame)return;view.setState(frame);state=frame;selected=view.selected;updateSelection(true);$('time').textContent=(frame.time_fs/1000).toFixed(3)+' ps';$('live-badge').textContent=`轨迹回放 ${i+1} / ${replayFrames.length}`;$('step-count').textContent=Number(frame.steps).toLocaleString()+' 个固定物理步';$('measured-temperature').textContent=Number(frame.measured_temperature).toFixed(1)+' K';for(const [id,key]of [['kinetic','kinetic_kcal_mol'],['potential','potential_kcal_mol'],['total-energy','total_kcal_mol'],['energy-change','energy_change_kcal_mol']])$(id).textContent=frame[key]==null?'不适用':Number(frame[key]).toFixed(5);$('dof').textContent=frame.degrees_of_freedom;$('backend-label').textContent='轨迹回放 · '+frame.model;$('model-note').textContent='仅展示导出的观测帧，不重新积分；实时模拟已暂停。';$('simulation-parameters').textContent=`${frame.box_size} Å 模拟盒 · 固定步长 ${frame.timestep_fs} fs`;$('molecule-count').textContent=frame.molecules.length;$('atom-count').textContent=frame.atoms.length;}
$('replay-file').addEventListener('change',guard(async()=>{
  const file=$('replay-file').files[0];if(!file)return;if(file.size>50*1024*1024)throw new Error('回放文件不能超过 50 MB');
  const data=JSON.parse(await file.text());
  const validatedFrames=validateTrajectory(data);
  if(!await perform({action:'playback',running:false}))return;
  recording=false;$('record').textContent='● 开始记录轨迹';replaying=true;replayFrames=validatedFrames;
  $('replay-frame').hidden=false;$('replay-frame').max=replayFrames.length-1;$('replay-frame').value=0;$('return-live').hidden=false;view.select(null);showReplay(0);
}));
$('replay-frame').addEventListener('input',()=>showReplay(Number($('replay-frame').value)));
$('return-live').addEventListener('click',()=>{replaying=false;revision=-1n;$('replay-frame').hidden=true;$('return-live').hidden=true;replayFrames=[];toast('已返回实时模拟，保持暂停');});
