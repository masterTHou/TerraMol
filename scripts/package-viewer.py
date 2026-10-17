"""Package the standalone Windows x64 EXE and beginner entry point (no DLL/runtime needed)."""
import hashlib
from pathlib import Path
import re
import struct
import zipfile

root=Path(__file__).resolve().parents[1]
exe=root/'TerraMol.exe'
data=exe.read_bytes()
offset=struct.unpack_from('<I',data,0x3c)[0]
if data[:2]!=b'MZ' or data[offset:offset+4]!=b'PE\0\0' or struct.unpack_from('<H',data,offset+4)[0]!=0x8664:
    raise RuntimeError('A Windows x64 viewer is required; run scripts/build-viewer.ps1 first')
version=re.search(r'^version\s*=\s*"([^"]+)"',(root/'Cargo.toml').read_text(),re.M)[1]
out=root/'artifacts/release'
out.mkdir(parents=True,exist_ok=True)
archive=out/f'TerraMol-Viewer-v{version}-windows-x64.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
    for source in ['TerraMol.exe','启动TerraMol.cmd','新手开始.md','LICENSE']:
        z.write(root/source,source)
    z.writestr('README.txt','TerraMol portable molecular viewer\n\n'
        '1. EXTRACT ALL files to a folder.\n2. Double-click TerraMol.exe or the .cmd launcher.\n'
        '3. Your default browser opens with 50 moving water molecules.\n'
        'No Unity, Rust, Python, Node.js or Internet connection is needed.\n'
        'Use the top-right Exit button to stop. An unused session exits after 3 minutes.\n\n'
        'Native rigid translation + approximate bounding-sphere collisions. Not atomistic MD.\n'
        'Source and scientific limitations: https://github.com/masterTHou/TerraMol\n')
with zipfile.ZipFile(archive) as z:
    assert z.testzip() is None
digest=hashlib.sha256(archive.read_bytes()).hexdigest()
(out/'VIEWER-SHA256SUMS.txt').write_text(f'{digest}  {archive.name}\n',encoding='ascii')
print(archive)
print('SHA256:',digest)
