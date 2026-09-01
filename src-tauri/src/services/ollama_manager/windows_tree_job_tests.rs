use super::ollama_tree_job::OllamaTreeJob;
use crate::services::owned_process::OwnedProcess;
use std::os::windows::io::AsRawHandle;
use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn ollama_job_termination_removes_root_and_model_runner() {
    let python = crate::services::test_runtime::python().expect("runtime Python de test");
    let temp = tempfile::tempdir().expect("temp root");
    let gate = temp.path().join("start.gate");
    let pids = temp.path().join("tree.pids");
    let script = "import os,pathlib,subprocess,sys,time; gate=pathlib.Path(sys.argv[1]);\nwhile not gate.exists(): time.sleep(.01)\nchild=subprocess.Popen([sys.executable,'-c','import time;time.sleep(120)']); target=pathlib.Path(sys.argv[2]); pending=target.with_suffix('.tmp'); pending.write_text(f'{os.getpid()},{child.pid}'); os.replace(pending,target); time.sleep(120)";
    let mut root = Command::new(python)
        .args(["-c", script])
        .arg(&gate)
        .arg(&pids)
        .spawn()
        .expect("start Ollama tree fixture");
    let job = OllamaTreeJob::create().expect("create Ollama job");

    OwnedProcess::admit_suspended_handle(root.as_raw_handle())
        .expect("admit root to Beaver global job");
    job.assign_process(root.as_raw_handle())
        .expect("nest root in Ollama job");
    std::fs::write(&gate, b"start").expect("open fixture gate");

    let started_deadline = Instant::now() + Duration::from_secs(5);
    while !pids.exists() && Instant::now() < started_deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let ids = std::fs::read_to_string(&pids).expect("root and runner started");
    let ids = ids
        .split(',')
        .map(|pid| pid.parse::<u32>().expect("numeric pid"))
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);

    job.terminate_and_wait(Instant::now() + Duration::from_secs(5))
        .expect("terminate complete Ollama tree");
    root.wait().expect("reap root fixture");

    let gone_deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut system = sysinfo::System::new();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        if ids
            .iter()
            .all(|pid| system.process(sysinfo::Pid::from_u32(*pid)).is_none())
        {
            break;
        }
        assert!(
            Instant::now() < gone_deadline,
            "an Ollama model runner survived its private job termination"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
