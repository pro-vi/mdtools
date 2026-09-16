#!/usr/bin/env -S /usr/bin/python3 -I -S
"""Private Bash launcher and macOS process identity, using only the standard library.

The controller copies this file beside its private shell.json. Agent commands
cannot read or execute this Python entry point through the native profile.
"""

from __future__ import annotations

import ctypes
from dataclasses import asdict, dataclass
import errno
import fcntl
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


class ContainmentError(RuntimeError):
    """Containment or exact process ownership could not be established."""


class _BsdInfo(ctypes.Structure):
    # Apple's SDK sys/proc_info.h, PROC_PIDTBSDINFO. The returned byte count is
    # checked so a changed ABI fails closed instead of inventing a birth time.
    _fields_ = [(name, ctypes.c_uint32) for name in (
        'flags', 'status', 'xstatus', 'pid', 'ppid', 'uid', 'gid', 'ruid',
        'rgid', 'svuid', 'svgid', 'reserved')] + [
        ('comm', ctypes.c_char * 16), ('name', ctypes.c_char * 32),
        *[(name, ctypes.c_uint32) for name in ('nfiles', 'pgid', 'jobc', 'dev', 'tpgid')],
        ('nice', ctypes.c_int32), ('start_seconds', ctypes.c_uint64),
        ('start_microseconds', ctypes.c_uint64)]


@dataclass(frozen=True)
class ProcessIdentity:
    pid: int
    seconds: int
    microseconds: int

    def __post_init__(self) -> None:
        if any(type(value) is not int or value < 0 for value in asdict(self).values()) or self.pid < 2:
            raise ContainmentError('invalid process identity')


@dataclass(frozen=True)
class ProcessState:
    identity: ProcessIdentity
    parent_pid: int
    group_id: int
    session_id: int
    status: int


def process_state(pid: int) -> ProcessState | None:
    if sys.platform != 'darwin':
        raise ContainmentError('native containment requires macOS')
    library = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
    library.proc_pidinfo.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_uint64, ctypes.c_void_p, ctypes.c_int]
    library.proc_pidinfo.restype = ctypes.c_int
    info = _BsdInfo()
    count = library.proc_pidinfo(pid, 3, 0, ctypes.byref(info), ctypes.sizeof(info))
    if count != ctypes.sizeof(info):
        if count == 0 and ctypes.get_errno() == errno.ESRCH:
            return None
        raise ContainmentError('cannot establish process identity')
    try:
        session_id = os.getsid(pid)
    except ProcessLookupError:
        return None
    return ProcessState(ProcessIdentity(info.pid, info.start_seconds, info.start_microseconds),
                        info.ppid, info.pgid, session_id, info.status)


def _private_json(path: Path, payload: object) -> None:
    with path.open('x', encoding='utf-8') as handle:
        path.chmod(0o600)
        json.dump(payload, handle, sort_keys=True)
        handle.write('\n')
        handle.flush()
        os.fsync(handle.fileno())


def _signal_identity(identity: ProcessIdentity, sig: int) -> None:
    state = process_state(identity.pid)
    if state is None or state.status == 5:  # SZOMB; no executable writer remains.
        return
    if state.identity != identity:
        raise ContainmentError('process identity changed; refusing signal')
    try:
        os.kill(identity.pid, sig)
    except ProcessLookupError:
        return


def _session_members(sessions: set[int]) -> list[ProcessState]:
    # Read fixed kernel metadata only, never commands or process environments.
    listed = subprocess.run(['/bin/ps', '-axo', 'pid='], capture_output=True, check=True)
    members = []
    for token in listed.stdout.split():
        pid = int(token)
        if pid < 2:
            continue
        try:
            sid = os.getsid(pid)
        except ProcessLookupError:
            continue
        if sid in sessions:
            state = process_state(pid)
            if state is not None and state.session_id in sessions and state.status != 5:
                members.append(state)
    return members


class OwnedShells:
    """Close launch admission and stop only the parent and registered sessions.

Each permitted Bash executable stays in its launcher-created session, including
job-control process groups. No permitted task executable creates a new session.
The native profile denies signals to other processes; cleanup is controller-owned.
"""

    def __init__(self, registry: Path, owner: ProcessIdentity) -> None:
        if owner.pid == os.getpid():
            raise ContainmentError('controller cannot own itself as a child')
        state = process_state(owner.pid)
        if state is not None and (state.identity != owner or state.session_id != owner.pid):
            raise ContainmentError('owner must be the exact new-session child')
        self.registry, self.owner = registry, owner

    def registrations(self) -> list[dict[str, object]]:
        records = []
        for path in sorted(self.registry.glob('shell-*.json')):
            if path.is_symlink():
                raise ContainmentError('symlink in shell registrations')
            raw = json.loads(path.read_bytes())
            if type(raw) is not dict or set(raw) != {'identity', 'parent', 'argv', 'cwd', 'profile_sha256'}:
                raise ContainmentError('malformed shell registration')
            child = ProcessIdentity(**raw['identity'])
            parent = ProcessIdentity(**raw['parent'])
            if parent != self.owner or child == self.owner:
                raise ContainmentError('foreign shell registration')
            expected = f'shell-{child.pid}-{child.seconds}-{child.microseconds}.json'
            if path.name != expected:
                raise ContainmentError('shell registration name mismatch')
            records.append(raw)
        return records

    def stop(self) -> None:
        # The same lock brackets recording + setsid in the launcher. Closing it
        # prevents an unrecorded session from appearing after this snapshot.
        with (self.registry / '.lock').open('r+b') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            closed = self.registry / 'closed'
            if not closed.exists():
                _private_json(closed, {'owner': asdict(self.owner)})
            _signal_identity(self.owner, signal.SIGSTOP)
            records = self.registrations()
        roots = [self.owner, *(ProcessIdentity(**raw['identity']) for raw in records)]
        sessions = {identity.pid for identity in roots}
        deadline = time.monotonic() + 3
        while True:
            # A reused root PID is not ownership, even if no descendants remain.
            for identity in roots:
                state = process_state(identity.pid)
                if state is not None and state.identity != identity:
                    raise ContainmentError('registered process identity changed')
            members = _session_members(sessions)
            if not members:
                return
            for state in members:
                _signal_identity(state.identity, signal.SIGSTOP)
            # Once stopped, processes cannot fork between the next scan and kill.
            stopped = _session_members(sessions)
            if all(state.status == 4 for state in stopped):  # SSTOP
                for state in stopped:
                    _signal_identity(state.identity, signal.SIGKILL)
            if time.monotonic() >= deadline:
                raise ContainmentError('owned processes remain; preserve workspace')
            time.sleep(0.01)


def launch_shell(config_path: Path, argv: list[str]) -> None:
    config = json.loads(config_path.read_bytes())
    if type(config) is not dict or set(config) != {'profile', 'profile_sha256', 'registry', 'bash', 'environment'}:
        raise ContainmentError('invalid private launcher configuration')
    if not ((len(argv) == 2 and argv[0] == '-c') or
            (len(argv) == 3 and argv[:2] in (['-c', '-l'], ['-l', '-c'])) or argv == ['--version']):
        raise ContainmentError('unverified Claude shell argv')
    profile = Path(config['profile'])
    if profile.is_symlink() or hashlib.sha256(profile.read_bytes()).hexdigest() != config['profile_sha256']:
        raise ContainmentError('missing or changed native profile')
    own, parent = process_state(os.getpid()), process_state(os.getppid())
    if own is None or parent is None:
        raise ContainmentError('missing launcher ownership')
    registry = Path(config['registry'])
    with (registry / '.lock').open('r+b') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if (registry / 'closed').exists():
            raise ContainmentError('shell launch admission closed')
        identity = own.identity
        target = registry / f'shell-{identity.pid}-{identity.seconds}-{identity.microseconds}.json'
        _private_json(target, {'identity': asdict(identity), 'parent': asdict(parent.identity),
                              'argv': argv, 'cwd': os.getcwd(), 'profile_sha256': config['profile_sha256']})
        if own.session_id != own.identity.pid:
            os.setsid()
    os.execve('/usr/bin/sandbox-exec', ['/usr/bin/sandbox-exec', '-f', str(profile),
              config['bash'], '--noprofile', '--norc', *argv], config['environment'])


if __name__ == '__main__':
    try:
        launch_shell(Path(__file__).with_name('shell.json'), sys.argv[1:])
    except (OSError, ValueError, TypeError, ContainmentError):
        print('mdtools: native shell admission failed', file=sys.stderr)
        raise SystemExit(78)
