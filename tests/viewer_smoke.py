"""Real HTTP + Rust executable acceptance test, with no browser or third-party Python packages."""
import concurrent.futures
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

with tempfile.TemporaryDirectory(prefix='terramol-viewer-') as temp:
    ready = Path(temp)/'ready.json'
    process = subprocess.Popen([str(Path(sys.argv[1]).resolve()), '--no-browser', '--ready-file', str(ready)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    try:
        deadline=time.monotonic()+20
        while not ready.exists():
            if process.poll() is not None:raise RuntimeError(process.stderr.read().decode(errors='replace'))
            if time.monotonic()>deadline:raise TimeoutError('Viewer did not start')
            time.sleep(.05)
        url=json.loads(ready.read_text())['url']
        def get(path='api/state', headers=None):
            with urllib.request.urlopen(urllib.request.Request(url+path,headers=headers or {}),timeout=5) as response:
                return response.read(),response.headers
        def action(name,**values):
            data=json.dumps({'action':name,**values}).encode()
            with urllib.request.urlopen(urllib.request.Request(url+'api/action',data=data,headers={'Content-Type':'application/json'}),timeout=5) as response:
                return json.load(response)
        page,headers=get('')
        assert '分子探索空间' in page.decode()
        assert '分子探索空间' in get('?renderer=canvas')[0].decode()
        assert "frame-ancestors 'none'" in headers['Content-Security-Policy']
        for name in ['app.js','renderer.js','style.css','favicon.svg']:assert len(get(name)[0])>100
        paused=action('playback',running=False)
        assert len(paused['atoms'])==150 and len(paused['molecules'])==50
        assert isinstance(paused['atoms'][0]['molecule_id'],str)
        time.sleep(.08)
        assert json.loads(get()[0])['steps']==paused['steps']
        # Network input is validated before it can mutate scientific state.
        for payload in [{'action':'speed','value':999},{'action':'spawn','kind':0,'count':100000},{'action':'clear','unknown':True}]:
            try:
                req=urllib.request.Request(url+'api/action',data=json.dumps(payload).encode(),headers={'Content-Type':'application/json'})
                urllib.request.urlopen(req,timeout=5)
                raise AssertionError('Invalid action accepted')
            except urllib.error.HTTPError as e:assert e.code in (400,409)
        for headers in [{'Origin':'https://example.org'},{'Host':'example.org'}]:
            try:get(headers=headers);raise AssertionError('Foreign origin/host accepted')
            except urllib.error.HTTPError as e:assert e.code==403
        try:
            urllib.request.urlopen(urllib.parse.urljoin(url,'/api/state'),timeout=5)
            raise AssertionError('Missing capability token accepted')
        except urllib.error.HTTPError as e:assert e.code==404
        action('reset',kind=1,seed=42,kelvin=300)
        a=action('playback',running=False)
        assert len(a['atoms'])==250
        action('spawn',kind=2,count=1)
        b=json.loads(get()[0]);assert len(b['atoms'])==262
        selected=b['molecules'][-1]['id'];action('velocity',id=selected,value=[.02,0,0])
        b=action('remove',id=selected);assert len(b['atoms'])==250
        assert all(edge['a']<250 and edge['b']<250 for edge in b['bonds'])
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            states=list(pool.map(lambda _:json.loads(get()[0]),range(12)))
        assert all(len(s['atoms'])==250 for s in states)
        action('thermal',kelvin=400)
        b=action('clear');assert not b['atoms'] and not b['bonds']
        action('shutdown');assert process.wait(timeout=5)==0
        print('Portable viewer: real Rust HTTP lifecycle, actions, assets, concurrency and access checks OK')
    finally:
        if process.poll() is None:process.terminate();process.wait(timeout=5)
        process.stderr.close()

# Closing the last tab eventually releases the listener/world without a persistent service.
with tempfile.TemporaryDirectory(prefix='terramol-idle-') as temp:
    process=subprocess.Popen([str(Path(sys.argv[1]).resolve()),'--no-browser','--idle-timeout-seconds','1','--ready-file',str(Path(temp)/'ready.json')],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
    try:
        assert process.wait(timeout=6)==0
    finally:
        if process.poll() is None:process.terminate();process.wait(timeout=5)
        process.stderr.close()
print('Portable viewer: idle session exits and releases resources OK')
