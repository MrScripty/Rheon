"""Linux campaign supervision; sampled RSS is not a continuous memory bound.

The supervisor stays outside a fresh controller session. Every fixed controller
child inherits that session. Monotonic deadlines cover spawn, Git, hashes,
controller work and cleanup; the outer alarm is a final fail-closed backstop.
No global limits/cgroups are changed. Fixed producer quotas cap retained writes.
"""
import json, math, os, resource, selectors, signal, subprocess, time
from dataclasses import dataclass
from pathlib import Path

TOTAL_SECONDS=1080.; CLEANUP_SECONDS=3.; SAMPLE_SECONDS=.05
RSS_THRESHOLD=256*1024*1024; OUTPUT_CAP=4*1024*1024
SOURCE_CAP=64*1024; COMPARISON_CAP=64*1024; LOG_CAP=64*1024
SUMMARY_CAP=256*1024; RECEIPT_CAP=64*1024; NATIVE_CAP=1024*1024
QUOTAS={'source':SOURCE_CAP,'native':3*NATIVE_CAP,'comparison':3*COMPARISON_CAP,
        'child_logs':6*LOG_CAP,'summary':SUMMARY_CAP,
        'controller_log':LOG_CAP,'guard_receipt':RECEIPT_CAP}
assert sum(QUOTAS.values())==OUTPUT_CAP

class GuardViolation(RuntimeError): pass

EVENT_PREFIX=b'RHEON_GUARD_EVENT '
class ChildDeadlineProtocol:
    """Bounded launch/done messages let the supervisor cover blocked spawn."""
    def __init__(self,work_end):
        self.pending=bytearray();self.truncated=False;self.deadline=None;self.work_end=work_end
    def feed(self,chunk):
        for piece in chunk.splitlines(keepends=True):
            if not self.truncated:
                self.pending.extend(piece[:513-len(self.pending)])
                if len(self.pending)>512:self.truncated=True
            if not piece.endswith(b'\n'):continue
            line=bytes(self.pending);self.pending.clear()
            truncated=self.truncated;self.truncated=False
            if not line.startswith(EVENT_PREFIX):continue
            if truncated:raise GuardViolation('controller deadline protocol cap')
            event=json.loads(line[len(EVENT_PREFIX):])
            if set(event)!= {'kind','time'} or not math.isfinite(event['time']):
                raise GuardViolation('invalid controller deadline protocol')
            if event['kind']=='launch':
                if self.deadline is not None or not event['time']<=self.work_end:
                    raise GuardViolation('overlapping/unbounded child deadline')
                self.deadline=event['time']
            elif event['kind']=='done':
                if self.deadline is None or event['time']>self.deadline:
                    raise GuardViolation('child deadline exceeded before completion')
                self.deadline=None
            else:raise GuardViolation('unknown controller deadline event')

def publish_child_event(kind,when):
    if os.environ.get('RHEON_CAMPAIGN_GUARDED')!='1':
        raise GuardViolation('bounded child requires campaign supervisor')
    data=EVENT_PREFIX+json.dumps({'kind':kind,'time':when}).encode()+b'\n'
    if len(data)>512 or os.write(1,data)!=len(data):
        raise GuardViolation('controller deadline publication failed')

class BoundedWriter:
    """Reject an entire chunk before a write could exceed this reserved slot."""
    def __init__(self,path,cap):
        if type(cap) is not int or cap<1:raise GuardViolation('invalid write quota')
        self.file=Path(path).open('xb',buffering=0);self.cap=cap;self.written=0
    def write(self,data):
        if not isinstance(data,bytes):raise TypeError('bounded writer requires bytes')
        if len(data)>self.cap-self.written:raise GuardViolation('output write quota')
        view=memoryview(data)
        while view:
            count=self.file.write(view)
            if not count:raise GuardViolation('output write made no progress')
            self.written+=count;view=view[count:]
        return len(data)
    def close(self):self.file.close()
    def __enter__(self):return self
    def __exit__(self,*_):self.close()

def write_reserved(path,data,cap):
    with BoundedWriter(path,cap) as stream:stream.write(data)

def proc_info(pid):
    with open(f'/proc/{pid}/stat','rb') as stream:raw=stream.read(4097)
    if len(raw)>4096:raise GuardViolation('oversized proc stat')
    parts=raw.rsplit(b')',1)[1].split()
    return {'pid':pid,'ppid':int(parts[1]),'pgrp':int(parts[2]),'session':int(parts[3]),
            'start':int(parts[19]),'rss':int(parts[21])*os.sysconf('SC_PAGE_SIZE'),
            'state':parts[0].decode('ascii')}

def snapshot(controller_pid,known):
    infos={}
    for entry in Path('/proc').iterdir():
        if not entry.name.isdecimal():continue
        try:info=proc_info(int(entry.name))
        except (FileNotFoundError,ProcessLookupError):continue
        infos[info['pid']]=info
    owned={pid for pid,info in infos.items()
           if info['session']==controller_pid or info['pgrp']==controller_pid
           or (pid,info['start']) in known}
    owned.add(controller_pid)
    changed=True
    while changed:
        added={pid for pid,info in infos.items() if info['ppid'] in owned}-owned
        changed=bool(added);owned.update(added)
    return {pid:infos[pid] for pid in owned if pid in infos}

@dataclass(frozen=True)
class Limits:
    seconds:float=TOTAL_SECONDS
    cleanup:float=CLEANUP_SECONDS
    sample:float=SAMPLE_SECONDS
    rss:int=RSS_THRESHOLD
    # Fixed campaign has one controller plus one child. Synthetic tests may
    # explicitly use more members to verify descendant cleanup.
    members:int=2
    def validate(self):
        if not all(math.isfinite(x) and x>0 for x in (self.seconds,self.cleanup,self.sample)) or self.cleanup>=self.seconds:
            raise GuardViolation('invalid guard timing limits')
        if type(self.rss) is not int or self.rss<1 or type(self.members) is not int or self.members<1:
            raise GuardViolation('invalid guard memory/member limits')

class ControllerProcess:
    """Known PID before any exec-readiness wait; no Popen startup error pipe."""
    def __init__(self,pid,reader):
        self.pid=pid;self.stdout=os.fdopen(reader,'rb',buffering=0)
        self.pidfd=None;self.returncode=None
    def poll(self):
        if self.returncode is None:
            pid,status=os.waitpid(self.pid,os.WNOHANG)
            if pid:self.returncode=os.waitstatus_to_exitcode(status)
        return self.returncode
    def wait(self,timeout):
        until=time.monotonic()+timeout
        while self.poll() is None:
            if time.monotonic()>=until:raise subprocess.TimeoutExpired('controller',timeout)
            time.sleep(min(.005,max(0.,until-time.monotonic())))
        return self.returncode

def supervise(command,out,*,limits=Limits(),started=None,env=None,prepare=None):
    """Return receipt; command must be a fixed trusted controller, not a shell."""
    limits.validate();started=time.monotonic() if started is None else started
    end=started+limits.seconds;work_end=end-limits.cleanup
    process=None;owned_fds={};known=set();peak=0;members_peak=0;samples=0
    violation=None;returncode=None;survivors=[];unreaped=[];cleanup_errors=[]
    previous_handler=signal.getsignal(signal.SIGALRM)
    previous_cancellation={s:signal.getsignal(s) for s in (signal.SIGTERM,signal.SIGINT)}
    in_cleanup=False
    previous_timer=signal.getitimer(signal.ITIMER_REAL)
    if previous_timer[0] or previous_timer[1]:raise GuardViolation('existing alarm cannot be replaced')
    def kill_owned():
        if process is not None:
            try:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            except OSError as error:cleanup_errors.append(str(error))
            if process.pidfd is not None:
                try:signal.pidfd_send_signal(process.pidfd,signal.SIGKILL)
                except ProcessLookupError:pass
                except OSError as error:cleanup_errors.append(str(error))
        for fd in owned_fds.values():
            try:signal.pidfd_send_signal(fd,signal.SIGKILL)
            except ProcessLookupError:pass
            except OSError as error:cleanup_errors.append(str(error))
    def emergency(_signum,_frame):
        # No waits, file I/O or controller cooperation at the final deadline.
        kill_owned();os._exit(124)
    def cancelled(signum,_frame):
        kill_owned()
        if not in_cleanup:raise GuardViolation(f'supervisor cancellation signal {signum}')
    for signum in previous_cancellation:signal.signal(signum,cancelled)
    signal.signal(signal.SIGALRM,emergency)
    signal.setitimer(signal.ITIMER_REAL,max(.000001,end-time.monotonic()))
    log=None;selector=None;created_out=False;session_established=False
    try:
        if prepare is not None:command,out=prepare()
        out=Path(out)
        if time.monotonic()>=work_end:raise GuardViolation('aggregate work deadline')
        out.mkdir(parents=False,exist_ok=False)
        created_out=True
        log=BoundedWriter(out/'controller.log',LOG_CAP)
        child_env=os.environ.copy();child_env.update(env or {})
        child_env.update(RHEON_CAMPAIGN_WORK_DEADLINE=repr(work_end),
                         RHEON_CAMPAIGN_GUARDED='1',PYTHONDONTWRITEBYTECODE='1')
        # Mask cancellation only over fork and PID/pidfd publication. There is
        # no wait for exec readiness while signals are masked. The pidfd also
        # covers the child before it has established its private session.
        reader,writer=os.pipe()
        old_mask=signal.pthread_sigmask(signal.SIG_BLOCK,{signal.SIGTERM,signal.SIGINT,signal.SIGALRM})
        try:
            pid=os.fork()
            if pid==0:
                try:
                    for signum in (signal.SIGTERM,signal.SIGINT,signal.SIGALRM):signal.signal(signum,signal.SIG_DFL)
                    os.close(reader);os.setsid();os.dup2(writer,1);os.dup2(writer,2)
                    os.close(writer)
                    signal.pthread_sigmask(signal.SIG_SETMASK,old_mask)
                    os.execvpe(command[0],command,child_env)
                except BaseException:
                    os.write(2,b'controller exec failed\n');os._exit(127)
            process=ControllerProcess(pid,reader);reader=None
            try:process.pidfd=os.pidfd_open(pid)
            except ProcessLookupError:pass # already exited, poll() will reap
        finally:
            if reader is not None:os.close(reader)
            os.close(writer)
            signal.pthread_sigmask(signal.SIG_SETMASK,old_mask)
        selector=selectors.DefaultSelector();selector.register(process.stdout,selectors.EVENT_READ)
        next_sample=time.monotonic()
        protocol=ChildDeadlineProtocol(work_end)
        eof=False
        while True:
            now=time.monotonic()
            if now>=work_end:raise GuardViolation('aggregate work deadline')
            if protocol.deadline is not None and now>=protocol.deadline:
                raise GuardViolation('supervised child deadline including spawn')
            if now>=next_sample:
                group=snapshot(process.pid,known)
                for pid,info in group.items():
                    identity=(pid,info['start']);known.add(identity)
                    if identity not in owned_fds:
                        try:fd=os.pidfd_open(pid)
                        except ProcessLookupError:continue
                        # Reject a reused PID between discovery and pidfd open.
                        try:after=proc_info(pid)
                        except (FileNotFoundError,ProcessLookupError):os.close(fd);continue
                        if after['start']!=info['start']:os.close(fd);continue
                        owned_fds[identity]=fd
                rss=sum(info['rss'] for info in group.values())+proc_info(os.getpid())['rss']
                live=[i for i in group.values() if i['state']!= 'Z']
                if any(i['pid']==process.pid and i['pgrp']==process.pid and i['session']==process.pid for i in live):
                    session_established=True
                samples+=1;peak=max(peak,rss);members_peak=max(members_peak,len(live))
                if any((i['pgrp']!=process.pid or i['session']!=process.pid)
                       and (i['pid']!=process.pid or session_established) for i in live):
                    raise GuardViolation('descendant escaped controller session/group')
                if len(live)>limits.members:raise GuardViolation('process-group member threshold')
                if rss>limits.rss:raise GuardViolation('sampled whole-group RSS threshold')
                next_sample=now+limits.sample
            rc=process.poll()
            if rc is not None and eof: returncode=rc;break
            timeout=max(0.,min(next_sample,work_end,protocol.deadline or work_end)-time.monotonic())
            for key,_ in selector.select(timeout):
                chunk=os.read(key.fileobj.fileno(),8192)
                if chunk:log.write(chunk);protocol.feed(chunk)
                else:selector.unregister(key.fileobj);eof=True
            # A descendant holding stdout open cannot turn controller exit into
            # an unlimited pipe drain. Refuse and clean the whole known group.
            if rc is not None and not eof:
                group=snapshot(process.pid,known)
                if any(i['pid']!=process.pid and i['state']!='Z' for i in group.values()):
                    raise GuardViolation('controller exited with live descendants')
        if returncode!=0:raise GuardViolation(f'controller exit {returncode}')
        if protocol.deadline is not None:raise GuardViolation('controller exited with unfinished child deadline')
    except BaseException as error:
        violation=f'{type(error).__name__}: {error}'
    finally:
        in_cleanup=True
        kill_owned()
        if process is not None:
            try:process.wait(timeout=max(.000001,min(1.,end-time.monotonic())))
            except subprocess.TimeoutExpired:violation=violation or 'controller cleanup timeout'
            if process.stdout is not None:process.stdout.close()
        if selector is not None:selector.close()
        if log is not None:log.close()
        if process is not None:
            group=snapshot(process.pid,known)
            survivors=[i['pid'] for i in group.values() if i['state']!='Z']
            unreaped=[i['pid'] for i in group.values() if i['state']=='Z']
            if survivors:violation=violation or 'live descendants after bounded cleanup'
        for fd in owned_fds.values():os.close(fd)
        owned_fds.clear()
        if process is not None and process.pidfd is not None:
            os.close(process.pidfd);process.pidfd=None
        if cleanup_errors:violation=violation or 'group cleanup signalling failed'
        receipt={'status':'refused' if violation else 'completed','reason':violation,
                 'controller_returncode':returncode,'aggregate_deadline_seconds':limits.seconds,
                 'controller_pid':None if process is None else process.pid,
                 'work_deadline_seconds':limits.seconds-limits.cleanup,'cleanup_reserve_seconds':limits.cleanup,
                 'rss_sample_threshold_bytes':limits.rss,'rss_sample_interval_seconds':limits.sample,
                 'rss_sampled_peak_bytes':peak,'rss_scope':'sum of supervisor and controller-group RSS; shared pages may count multiple times',
                 'continuous_rss_bound':False,'sampled_live_group_members_peak':members_peak,
                 'surviving_live_members':survivors,'unreaped_zombie_members':unreaped,
                 'cleanup_signalling_errors':cleanup_errors,
                 'sampled_distinct_descendants_including_controller':len(known),'samples':samples,
                 'output_cap_bytes':OUTPUT_CAP,'output_reserved_quotas':QUOTAS,
                 'elapsed_seconds_before_receipt':time.monotonic()-started,
                 'no_global_cgroup_or_security_changes':True}
        try:
            if created_out:write_reserved(Path(out)/'guard-receipt.json',(json.dumps(receipt,indent=2)+'\n').encode(),RECEIPT_CAP)
        finally:
            signal.setitimer(signal.ITIMER_REAL,0)
            signal.signal(signal.SIGALRM,previous_handler)
            for signum,handler in previous_cancellation.items():signal.signal(signum,handler)
        if time.monotonic()>=end:raise GuardViolation('aggregate deadline reached during cleanup/receipt')
    return receipt

def run_bounded(command,*,deadline,seconds,log_path=None,capture_cap=8192,cwd=None):
    """Single inherited-group child; every stdout/stderr chunk is bounded."""
    if time.monotonic()>=deadline:raise GuardViolation('aggregate controller deadline')
    finish=min(deadline,time.monotonic()+seconds)
    publish_child_event('launch',finish)
    sink=BoundedWriter(log_path,LOG_CAP) if log_path is not None else None
    captured=bytearray();process=None;selector=None;completed=False
    try:
        process=subprocess.Popen(command,cwd=cwd,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,bufsize=0)
        selector=selectors.DefaultSelector();selector.register(process.stdout,selectors.EVENT_READ)
        eof=False
        while not (eof and process.poll() is not None):
            remaining=finish-time.monotonic()
            if remaining<=0:raise GuardViolation('child/controller deadline')
            for key,_ in selector.select(min(.05,remaining)):
                chunk=os.read(key.fileobj.fileno(),8192)
                if not chunk:selector.unregister(key.fileobj);eof=True;continue
                if sink is not None:sink.write(chunk)
                else:
                    if len(chunk)>capture_cap-len(captured):raise GuardViolation('captured child output quota')
                    captured.extend(chunk)
        if process.returncode!=0:raise GuardViolation(f'child exit {process.returncode}')
        completed=True
        return bytes(captured)
    finally:
        if process is not None:
            if process.poll() is None:process.kill()
            try:process.wait(timeout=max(.000001,min(.5,deadline-time.monotonic())))
            except subprocess.TimeoutExpired:pass # outer supervisor owns cleanup
            process.stdout.close()
        if selector is not None:selector.close()
        if sink is not None:sink.close()
        if completed:publish_child_event('done',time.monotonic())

def controller_deadline():
    if os.environ.get('RHEON_CAMPAIGN_GUARDED')!='1' or os.getsid(0)!=os.getpid() or os.getpgrp()!=os.getpid():
        raise GuardViolation('controller requires the supervising session')
    value=float(os.environ['RHEON_CAMPAIGN_WORK_DEADLINE'])
    if not math.isfinite(value) or value<=time.monotonic():raise GuardViolation('invalid controller deadline')
    # Local inherited regular-file backstop only. The aggregate is enforced by
    # the explicit producer quotas; this does not replace pipe/native guards.
    resource.setrlimit(resource.RLIMIT_FSIZE,(NATIVE_CAP,NATIVE_CAP))
    return value
