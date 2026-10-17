"""Pack exact corresponding sources for the combined GPL v2 viewer distribution."""
from pathlib import Path
import hashlib
import tarfile
import subprocess
import io
import sys
root=Path(__file__).resolve().parents[1]
source=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else root/'artifacts/lammps-min/lammps-stable_29Aug2024_update3'
out=root/'artifacts/release/TerraMol-v1.2.0-corresponding-source.tar.gz'
out.parent.mkdir(parents=True,exist_ok=True)
vendor=root/'artifacts/release/rust-vendor'
subprocess.run(['cargo','vendor','--locked','--respect-source-config','--versioned-dirs',str(vendor)],cwd=root,check=True,stdout=subprocess.DEVNULL)
with tarfile.open(out,'w:gz') as tar:
    tar.add(vendor,arcname='TerraMol/vendor')
    config=b'[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = "vendor"\n'
    info=tarfile.TarInfo('TerraMol/.cargo/config.toml');info.size=len(config);tar.addfile(info,io.BytesIO(config))
    for name in ['src','lib','cmake','LICENSE','README']:
        tar.add(source/name,arcname='lammps-stable_29Aug2024_update3/'+name)
    for name in ['src','viewer','include','python','unity/Assets','unity/Packages','unity/ProjectSettings','tests','examples','scripts','docs','third_party','assets','.github','Cargo.toml','Cargo.lock','README.md','LICENSE','CONTRIBUTING.md','RELEASE_NOTES.md','新手开始.md','启动TerraMol.cmd']:
        def clean(info):
            return None if info.name.endswith(('.dll','.pyc','.pdb','.exe')) or '__pycache__' in info.name else info
        tar.add(root/name,arcname='TerraMol/'+name,filter=clean)
print(out, out.stat().st_size,hashlib.sha256(out.read_bytes()).hexdigest())
