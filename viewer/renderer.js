// Rendering only. Atom positions, bonds, identities, colors and COM velocities come from Rust.
const add=(a,b)=>a.map((v,i)=>v+b[i]), sub=(a,b)=>a.map((v,i)=>v-b[i]);
const scale=(a,s)=>a.map(v=>v*s), dot=(a,b)=>a.reduce((v,x,i)=>v+x*b[i],0);
const cross=(a,b)=>[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
const length=a=>Math.hypot(...a), unit=a=>scale(a,1/(length(a)||1));
function multiply(a,b){const out=new Float32Array(16);for(let c=0;c<4;c++)for(let r=0;r<4;r++)for(let k=0;k<4;k++)out[c*4+r]+=a[k*4+r]*b[c*4+k];return out;}
function transform(m,p){return [0,1,2,3].map(r=>m[r]*p[0]+m[4+r]*p[1]+m[8+r]*p[2]+m[12+r]);}
function lookAt(eye,target){const z=unit(sub(eye,target)),x=unit(cross([0,1,0],z)),y=cross(z,x);return new Float32Array([x[0],y[0],z[0],0,x[1],y[1],z[1],0,x[2],y[2],z[2],0,-dot(x,eye),-dot(y,eye),-dot(z,eye),1]);}
function perspective(aspect){const f=1/Math.tan(Math.PI/8),near=.1,far=2000;return new Float32Array([f/aspect,0,0,0,0,f,0,0,0,0,(far+near)/(near-far),-1,0,0,2*far*near/(near-far),0]);}
function sphereModel(p,r){return new Float32Array([r,0,0,0,0,r,0,0,0,0,r,0,...p,1]);}
function rodModel(a,b,r){const delta=sub(b,a),y=unit(delta),x=unit(cross(Math.abs(y[1])>.95?[1,0,0]:[0,1,0],y)),z=cross(x,y),mid=scale(add(a,b),.5),h=length(delta)/2;return new Float32Array([...scale(x,r),0,...scale(y,h),0,...scale(z,r),0,...mid,1]);}
function sphereGeometry(){const vertices=[],indices=[],lat=12,lon=18;for(let i=0;i<=lat;i++){const p=i*Math.PI/lat;for(let j=0;j<=lon;j++){const t=j*Math.PI*2/lon,n=[Math.sin(p)*Math.cos(t),Math.cos(p),Math.sin(p)*Math.sin(t)];vertices.push(...n,...n);}}for(let i=0;i<lat;i++)for(let j=0;j<lon;j++){const a=i*(lon+1)+j,b=a+lon+1;indices.push(a,a+1,b,b,a+1,b+1);}return{vertices,indices};}
function cylinderGeometry(){const vertices=[],indices=[],n=12;for(let row=0;row<2;row++)for(let i=0;i<=n;i++){const t=i*Math.PI*2/n;vertices.push(Math.cos(t),row?1:-1,Math.sin(t),Math.cos(t),0,Math.sin(t));}for(let i=0;i<n;i++){const b=i+n+1;indices.push(i,b,i+1,b,b+1,i+1);}return{vertices,indices};}

class WebGLRenderer {
  constructor(canvas){
    const gl=canvas.getContext('webgl',{alpha:true,antialias:true});if(!gl)throw new Error('WebGL unavailable');this.gl=gl;
    const vertex=`attribute vec3 aPosition; attribute vec3 aNormal; uniform mat4 uModel; uniform mat4 uViewProjection; varying vec3 vNormal; varying vec3 vWorld; void main(){vec4 p=uModel*vec4(aPosition,1.0);vWorld=p.xyz;vNormal=mat3(uModel)*aNormal;gl_Position=uViewProjection*p;}`;
    const fragment=`precision mediump float;uniform vec3 uColor;uniform vec3 uEye;uniform float uFlat;varying vec3 vNormal;varying vec3 vWorld;void main(){vec3 n=normalize(vNormal);vec3 light=normalize(vec3(-0.4,0.8,0.6));vec3 view=normalize(uEye-vWorld);float diffuse=max(dot(n,light),0.0);float spec=pow(max(dot(reflect(-light,n),view),0.0),30.0);float rim=pow(1.0-max(dot(n,view),0.0),3.0);vec3 c=uColor*(0.30+0.68*diffuse)+vec3(spec*.30)+vec3(.10,.18,.20)*rim;c=mix(c,uColor,uFlat);gl_FragColor=vec4(c,1.0);}`;
    const shader=(type,source)=>{const s=gl.createShader(type);gl.shaderSource(s,source);gl.compileShader(s);if(!gl.getShaderParameter(s,gl.COMPILE_STATUS)){const message=gl.getShaderInfoLog(s);gl.deleteShader(s);throw new Error(message);}return s;};
    const vs=shader(gl.VERTEX_SHADER,vertex),fs=shader(gl.FRAGMENT_SHADER,fragment);this.program=gl.createProgram();gl.attachShader(this.program,vs);gl.attachShader(this.program,fs);gl.linkProgram(this.program);gl.deleteShader(vs);gl.deleteShader(fs);if(!gl.getProgramParameter(this.program,gl.LINK_STATUS))throw new Error('Shader linking failed');
    this.locations={};for(const key of ['uModel','uViewProjection','uColor','uEye','uFlat'])this.locations[key]=gl.getUniformLocation(this.program,key);
    this.position=gl.getAttribLocation(this.program,'aPosition');this.normal=gl.getAttribLocation(this.program,'aNormal');this.sphere=this.mesh(sphereGeometry());this.cylinder=this.mesh(cylinderGeometry());
    gl.enable(gl.DEPTH_TEST);gl.enable(gl.CULL_FACE);gl.clearColor(0,0,0,0);this.name='WebGL 3D';
  }
  mesh({vertices,indices}){const gl=this.gl,vbo=gl.createBuffer(),ibo=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,vbo);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array(vertices),gl.STATIC_DRAW);gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER,ibo);gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,new Uint16Array(indices),gl.STATIC_DRAW);return{vbo,ibo,count:indices.length};}
  draw(mesh,model,color,flat=false){const gl=this.gl;gl.bindBuffer(gl.ARRAY_BUFFER,mesh.vbo);gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER,mesh.ibo);gl.enableVertexAttribArray(this.position);gl.enableVertexAttribArray(this.normal);gl.vertexAttribPointer(this.position,3,gl.FLOAT,false,24,0);gl.vertexAttribPointer(this.normal,3,gl.FLOAT,false,24,12);gl.uniformMatrix4fv(this.locations.uModel,false,model);gl.uniform3fv(this.locations.uColor,color);gl.uniform1f(this.locations.uFlat,flat?1:0);gl.drawElements(gl.TRIANGLES,mesh.count,gl.UNSIGNED_SHORT,0);}
  frame(view){
    const gl=this.gl;gl.viewport(0,0,gl.canvas.width,gl.canvas.height);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);gl.useProgram(this.program);gl.uniformMatrix4fv(this.locations.uViewProjection,false,view.vp);gl.uniform3fv(this.locations.uEye,view.eye);
    for(const [a,b]of view.boxLines)this.draw(this.cylinder,rodModel(a,b,.018),[.10,.25,.31],true);
    for(const [a,b]of view.gridLines)this.draw(this.cylinder,rodModel(a,b,.009),[.065,.15,.20],true);
    const s=view.state;if(!s)return;
    for(const b of s.bonds){const a=s.atoms[b.a],z=s.atoms[b.b];this.draw(this.cylinder,rodModel(a.position,z.position,.085),a.molecule_id===view.selected?[.73,.76,.39]:[.42,.57,.64]);}
    for(const a of s.atoms){const e=view.elements.get(a.element),selected=a.molecule_id===view.selected;const color=selected?e.cpk.map((v,i)=>v*.45+[1,.82,.2][i]*.55):e.cpk;this.draw(this.sphere,sphereModel(a.position,e.vdw*.32*(selected?1.08:1)),color);}
    for(const [a,b]of view.arrowLines())this.draw(this.cylinder,rodModel(a,b,.055),[.96,.79,.25],true);
  }
  dispose(){const gl=this.gl;for(const m of [this.sphere,this.cylinder]){gl.deleteBuffer(m.vbo);gl.deleteBuffer(m.ibo);}gl.deleteProgram(this.program);}
}

class CanvasRenderer {
  constructor(canvas){this.ctx=canvas.getContext('2d');if(!this.ctx)throw new Error('Canvas unavailable');this.name='兼容 3D 投影';}
  frame(view){const ctx=this.ctx,w=ctx.canvas.width,h=ctx.canvas.height;ctx.clearRect(0,0,w,h);
    const line=(a,b,color,width)=>{const p=view.project(a),q=view.project(b);if(!p||!q)return;ctx.strokeStyle=color;ctx.lineWidth=width;ctx.beginPath();ctx.moveTo(p.x,p.y);ctx.lineTo(q.x,q.y);ctx.stroke();};
    for(const[a,b]of view.gridLines)line(a,b,'#17303d',1);for(const[a,b]of view.boxLines)line(a,b,'#264653',1);
    if(!view.state)return;const primitives=[];
    for(const b of view.state.bonds){const a=view.state.atoms[b.a],z=view.state.atoms[b.b],p=view.project(a.position),q=view.project(z.position);if(p&&q)primitives.push({depth:(p.depth+q.depth)/2,draw:()=>line(a.position,z.position,a.molecule_id===view.selected?'#ddcc6b':'#7494a3',Math.max(1.5,2.3*view.pixelRatio))});}
    for(const a of view.state.atoms){const p=view.project(a.position);if(!p)continue;const e=view.elements.get(a.element),selected=a.molecule_id===view.selected,r=Math.max(1,view.radiusPixels(e.vdw*.32,p.depth)),rgb=e.cpk.map(v=>Math.round(v*255));primitives.push({depth:p.depth,draw:()=>{const grad=ctx.createRadialGradient(p.x-r*.32,p.y-r*.4,r*.05,p.x,p.y,r);grad.addColorStop(0,`rgb(${rgb.map(v=>Math.min(255,v+70)).join(',')})`);grad.addColorStop(.6,`rgb(${rgb.join(',')})`);grad.addColorStop(1,`rgb(${rgb.map(v=>Math.round(v*.35)).join(',')})`);ctx.fillStyle=grad;ctx.beginPath();ctx.arc(p.x,p.y,r,0,Math.PI*2);ctx.fill();if(selected){ctx.strokeStyle='#edcf6b';ctx.lineWidth=2*view.pixelRatio;ctx.stroke();}}});}
    primitives.sort((a,b)=>b.depth-a.depth);for(const p of primitives)p.draw();for(const[a,b]of view.arrowLines())line(a,b,'#f5d76c',2*view.pixelRatio);
  }
  dispose(){}
}

export class MolecularView {
  constructor(canvas,onSelect,onRenderer){
    this.canvas=canvas;this.state=null;this.selected=null;this.elements=new Map();this.yaw=.67;this.pitch=.38;this.distance=80;this.box=40;this.pixelRatio=1;this.frames=0;this.fps=0;this.lastFps=performance.now();this.onSelect=onSelect;this.boxLines=[];this.gridLines=[];
    try{if(new URLSearchParams(location.search).get('renderer')==='canvas')throw new Error('Canvas compatibility mode requested');this.renderer=new WebGLRenderer(canvas);}catch(error){console.info('Using Canvas projection renderer:',error.message);if(canvas.getContext('2d')===null){const next=canvas.cloneNode();canvas.replaceWith(next);this.canvas=next;}this.renderer=new CanvasRenderer(this.canvas);}
    onRenderer(this.renderer.name);this.bindInput();this.fit();this.resizeObserver=new ResizeObserver(()=>this.resize());this.resizeObserver.observe(this.canvas);this.resize();this.alive=true;
    const frame=now=>{if(!this.alive)return;this.camera();this.renderer.frame(this);this.frames++;if(now-this.lastFps>750){this.fps=Math.round(this.frames*1000/(now-this.lastFps));this.frames=0;this.lastFps=now;}this.frameId=requestAnimationFrame(frame);};this.frameId=requestAnimationFrame(frame);
  }
  setState(s){this.state=s;this.elements=new Map(s.elements.map(e=>[e.number,e]));if(this.box!==s.box_size||!this.boxLines.length){this.box=s.box_size;this.buildBox();}if(this.selected&&!s.molecules.some(m=>m.id===this.selected))this.select(null);}
  select(id){this.selected=id;this.onSelect(id);}
  fitDistance(){const aspect=this.canvas.width/this.canvas.height;return (this.box*Math.sqrt(3)/2+2)/Math.sin(Math.atan(Math.tan(Math.PI/8)*Math.min(1,aspect)))*1.02;}
  fit(){this.yaw=.67;this.pitch=.38;this.zoom=1;this.distance=this.fitDistance();this.buildBox();}
  buildBox(){const b=this.box,p=[[0,0,0],[b,0,0],[b,b,0],[0,b,0],[0,0,b],[b,0,b],[b,b,b],[0,b,b]];this.boxLines=[[0,1],[1,2],[2,3],[3,0],[4,5],[5,6],[6,7],[7,4],[0,4],[1,5],[2,6],[3,7]].map(([a,z])=>[p[a],p[z]]);this.gridLines=[];for(let i=1;i<8;i++){const x=b*i/8;this.gridLines.push([[x,0,0],[x,0,b]],[[0,0,x],[b,0,x]]);}}
  resize(){const r=this.canvas.getBoundingClientRect();this.pixelRatio=Math.min(window.devicePixelRatio||1,2);this.canvas.width=Math.max(1,Math.round(r.width*this.pixelRatio));this.canvas.height=Math.max(1,Math.round(r.height*this.pixelRatio));this.distance=this.fitDistance()*(this.zoom||1);this.camera();}
  camera(){const center=[this.box/2,this.box/2,this.box/2];this.eye=add(center,scale([Math.cos(this.pitch)*Math.sin(this.yaw),Math.sin(this.pitch),Math.cos(this.pitch)*Math.cos(this.yaw)],this.distance));this.view=lookAt(this.eye,center);this.vp=multiply(perspective(this.canvas.width/this.canvas.height),this.view);}
  project(p){const clip=transform(this.vp,p);if(clip[3]<=.01)return null;const v=transform(this.view,p);return{x:(clip[0]/clip[3]*.5+.5)*this.canvas.width,y:(.5-clip[1]/clip[3]*.5)*this.canvas.height,depth:-v[2]};}
  radiusPixels(radius,depth){return radius*this.canvas.height/(2*Math.tan(Math.PI/8)*depth);}
  pick(x,y){if(!this.state)return;const rect=this.canvas.getBoundingClientRect(),px=(x-rect.left)*this.pixelRatio,py=(y-rect.top)*this.pixelRatio;let best=null,depth=Infinity;for(const a of this.state.atoms){const p=this.project(a.position);if(!p)continue;const radius=Math.max(6*this.pixelRatio,this.radiusPixels(this.elements.get(a.element).vdw*.32,p.depth));if(Math.hypot(p.x-px,p.y-py)<=radius&&p.depth<depth){best=a.molecule_id;depth=p.depth;}}this.select(best);}
  arrowLines(){const m=this.state?.molecules.find(m=>m.id===this.selected);if(!m)return[];const v=scale(m.velocity,200),n=length(v);if(n<.0001)return[];const direction=unit(v),end=add(m.center,v),side=unit(cross(direction,Math.abs(direction[1])>.95?[1,0,0]:[0,1,0])),back=sub(end,scale(direction,Math.min(1.5,n*.28))),wing=scale(side,Math.min(.65,n*.15));return[[m.center,end],[end,add(back,wing)],[end,sub(back,wing)]];}
  bindInput(){let drag=null;this.canvas.addEventListener('pointerdown',e=>{if(e.button!==0)return;drag={x:e.clientX,y:e.clientY,startX:e.clientX,startY:e.clientY,moved:false};this.canvas.setPointerCapture(e.pointerId);});this.canvas.addEventListener('pointermove',e=>{if(!drag)return;const dx=e.clientX-drag.x,dy=e.clientY-drag.y;if(Math.hypot(e.clientX-drag.startX,e.clientY-drag.startY)>4)drag.moved=true;if(drag.moved){this.yaw-=dx*.007;this.pitch=Math.max(-1.35,Math.min(1.35,this.pitch+dy*.007));}drag.x=e.clientX;drag.y=e.clientY;});this.canvas.addEventListener('pointerup',e=>{if(drag&&!drag.moved)this.pick(e.clientX,e.clientY);drag=null;});this.canvas.addEventListener('pointercancel',()=>{drag=null;});this.canvas.addEventListener('wheel',e=>{e.preventDefault();this.zoom=Math.max(.25,Math.min(4,(this.zoom||1)*Math.exp(e.deltaY*.001)));this.distance=this.fitDistance()*this.zoom;},{passive:false});}
  dispose(){this.alive=false;cancelAnimationFrame(this.frameId);this.resizeObserver.disconnect();this.renderer.dispose();}
}
