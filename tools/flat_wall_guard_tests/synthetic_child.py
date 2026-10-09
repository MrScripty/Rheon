"""Synthetic processes ONLY. No solver import, native launch or physical data."""
import sys
sys.dont_write_bytecode=True
import json,os,subprocess,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import flat_wall_campaign_guard as guard

def worker(mode):
    if mode=='okay':print('synthetic child',flush=True)
    elif mode=='sleep':time.sleep(20)
    elif mode=='flood':
        for _ in range(32):os.write(1,b'x'*8192)
        time.sleep(20)
    elif mode=='memory':
        block=bytearray(96*1024*1024)
        print(len(block),flush=True);time.sleep(20)
    elif mode=='descendant':
        child=subprocess.Popen([sys.executable,'-B',__file__,'worker','sleep'])
        print('DESCENDANT='+str(child.pid),flush=True);time.sleep(20)
    elif mode=='escape':
        os.setsid();print('ESCAPED='+str(os.getpid()),flush=True);time.sleep(20)
    elif mode=='file':
        p=Path(sys.argv[3]);p.parent.mkdir()
        with p.open('wb',buffering=0) as stream:
            for _ in range(130):stream.write(b'x'*8192)
    else:raise ValueError(mode)

def controller(mode,out):
    deadline=guard.controller_deadline()
    guard.write_reserved(out/'campaign-report.json',json.dumps({'controller_pid':os.getpid()}).encode(),guard.SUMMARY_CAP)
    if mode in ('hang','alarm'):time.sleep(20);return
    if mode=='controller_flood':
        for _ in range(32):os.write(1,b'x'*8192)
        time.sleep(20);return
    if mode=='postprocess':
        guard.run_bounded([sys.executable,'-B',__file__,'worker','okay'],deadline=deadline,seconds=.3,log_path=out/'n6.log')
        time.sleep(20);return
    if mode=='capture_flood':
        guard.run_bounded([sys.executable,'-B',__file__,'worker','flood'],deadline=deadline,seconds=1)
        return
    if mode=='spawn_hang':guard.subprocess.Popen=lambda *_args,**_kwargs:time.sleep(20)
    selected={'success':'okay','child_timeout':'sleep','spawn_hang':'sleep','log_flood':'flood','memory':'memory',
              'descendant':'descendant','escape':'escape','file':'file'}[mode]
    command=[sys.executable,'-B',__file__,'worker',selected]
    if mode=='file':command.append(str(out/'n6/acquisition.txt'))
    guard.run_bounded(command,deadline=deadline,seconds=.15 if mode in ('child_timeout','spawn_hang') else 3,
                      log_path=out/'n6.log')

if __name__=='__main__':
    role=sys.argv[1]
    if role=='worker':worker(sys.argv[2])
    elif role=='controller':controller(sys.argv[2],Path(sys.argv[3]))
    elif role=='supervisor':
        mode=sys.argv[2];out=Path(sys.argv[3]);seconds=float(sys.argv[4]);rss=int(sys.argv[5]);members=int(sys.argv[6])
        if mode=='alarm':guard.snapshot=lambda *_:time.sleep(20)
        receipt=guard.supervise([sys.executable,'-B',__file__,'controller',mode,str(out)],out,
            limits=guard.Limits(seconds=seconds,cleanup=.2,sample=.02,rss=rss,members=members))
        sys.exit(0 if receipt['status']=='completed' else 1)
    else:raise ValueError(role)
