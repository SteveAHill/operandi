//! Autosave survives a crash (scorecard row 63): the app is killed while a
//! backup or a save is being written, and everything on disk afterwards
//! still opens. The kill is real: the test runs itself again as a child
//! process that saves in a loop, and stops it with a hard kill (SIGKILL on
//! Unix, TerminateProcess on Windows) part-way through a write.
use super::*;
use operandi_core::{Face, Mesh};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Set in the child: the folder it saves into.
const CHILD: &str = "OPERANDI_AUTOSAVE_CRASH_CHILD";
const STEM: &str = "Crash";
const USER_FILE: &str = "Crash.opscene";

fn temp_dir(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "operandi-{tag}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

/// A scene big enough that each write takes a few tens of milliseconds,
/// so the kill usually lands inside one.
fn scene() -> Scene {
    let side = 250u32;
    let mut mesh = Mesh::default();
    for y in 0..=side {
        for x in 0..=side {
            mesh.vertices
                .push(glam::DVec3::new(f64::from(x) / 3., f64::from(y) / 7., 0.));
        }
    }
    for y in 0..side {
        for x in 0..side {
            let a = y * (side + 1) + x;
            mesh.faces.push(Face {
                vertices: vec![a, a + 1, a + side + 2, a + side + 1],
                uv: None,
                material: "Default".into(),
                smooth: true,
                part: None,
                material_label: None,
            });
        }
    }
    let mut scene = Scene::default();
    scene.new_mesh().unwrap();
    scene.items[0].mesh = mesh.into();
    scene
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    names
}
fn temps(dir: &Path) -> Vec<String> {
    names(dir)
        .into_iter()
        .filter(|n| n.ends_with(".operandi-tmp"))
        .collect()
}

/// The child: save the user file and write a backup, over and over, until
/// it is killed.
fn child_loop(dir: &Path) -> ! {
    let scene = scene();
    let user = dir.join(USER_FILE);
    loop {
        persistence::save(&user, &scene).unwrap();
        write_backup(dir, STEM, 3, &scene, Some(&user)).unwrap();
    }
}

/// Wait until `ready` holds, or fail after `limit`.
fn wait_for(limit: Duration, what: &str, mut ready: impl FnMut() -> bool) {
    let start = Instant::now();
    while !ready() {
        assert!(start.elapsed() < limit, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn a_crash_mid_save_leaves_every_file_openable() {
    if let Some(dir) = std::env::var_os(CHILD) {
        child_loop(Path::new(&dir));
    }
    let dir = temp_dir("autosave-crash");
    // This test's own name, without the crate's, so the child runs only it.
    let module = module_path!().split_once("::").unwrap().1;
    let name = format!("{module}::a_crash_mid_save_leaves_every_file_openable");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([name.as_str(), "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD, &dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    // Let it get going (two backups on disk), then kill it the moment a
    // write is half done. If no write is caught in time, kill it anyway:
    // the checks below must hold wherever the kill lands.
    wait_for(Duration::from_secs(120), "two backups", || {
        revisions(&dir, STEM).len() >= 2
    });
    let start = Instant::now();
    while temps(&dir).is_empty() && start.elapsed() < Duration::from_secs(10) {
        std::hint::spin_loop();
    }
    child.kill().unwrap();
    child.wait().unwrap();

    let expected = scene();
    // The user's own file is the old or the new save, never half of one.
    assert_eq!(
        persistence::load(&dir.join(USER_FILE)).unwrap(),
        expected,
        "the scene file opens after the crash"
    );
    // Every backup the folder lists opens whole.
    let found = revisions(&dir, STEM);
    assert!(found.len() >= 2, "{:?}", names(&dir));
    for (n, path) in &found {
        let back = persistence::load(path)
            .unwrap_or_else(|e| panic!("backup {n} doesn't open after the crash: {e}"));
        assert_eq!(back, expected, "backup {n}");
    }
    // Any half-written temporary is hidden and never taken for a backup.
    let leftovers = temps(&dir);
    for temp in &leftovers {
        assert!(temp.starts_with('.'), "{temp}");
        assert!(parse_backup(temp).is_none(), "{temp}");
    }
    // The next run's auto-save carries on numbering and opens.
    let last = found.last().unwrap().0;
    let next = write_backup(&dir, STEM, 3, &expected, Some(&dir.join(USER_FILE))).unwrap();
    assert_eq!(next, dir.join(backup_name(STEM, last + 1)));
    assert_eq!(persistence::load(&next).unwrap(), expected);
    // Leftovers stay while they're fresh (another copy of the app could be
    // writing one) and go once they're an hour old.
    // Only backups' temporaries are swept: a scene file's own leftover
    // (`.Crash.opscene.…`) sits in the user's folder, which cleanup never
    // touches.
    let backup_temps = leftovers
        .iter()
        .filter(|t| t.contains(".autosave-"))
        .count();
    let now = SystemTime::now();
    assert_eq!(remove_stale_temps(&dir, now), 0);
    let later = now + STALE_TEMP_AGE + Duration::from_secs(60);
    assert_eq!(remove_stale_temps(&dir, later), backup_temps);
    assert!(
        temps(&dir).iter().all(|t| !t.contains(".autosave-")),
        "{:?}",
        names(&dir)
    );
    assert!(dir.join(USER_FILE).exists());
    assert_eq!(revisions(&dir, STEM).len(), 3);
    fs::remove_dir_all(dir).unwrap();
}

/// Old-backup cleanup also sweeps a crash's leftover temporaries, and
/// nothing else: not the user's files, not other hidden files, not a
/// fresh temporary.
#[test]
fn old_backup_cleanup_sweeps_only_stale_backup_temps() {
    let dir = temp_dir("autosave-stale-temps");
    let scene = Scene::default();
    let backup = write_backup(&dir, "A", 5, &scene, None).unwrap();
    let stale = dir.join(".A.autosave-0002.opscene.4242-0.operandi-tmp");
    let other_tmp = dir.join(".Robot.opscene.4242-1.operandi-tmp");
    let hidden = dir.join(".A.autosave-notes");
    for path in [&stale, &other_tmp, &hidden] {
        fs::write(path, b"half").unwrap();
    }
    assert_eq!(remove_stale_temps(&dir, SystemTime::now()), 0);
    let later = SystemTime::now() + STALE_TEMP_AGE + Duration::from_secs(60);
    remove_old_backups(&dir, later);
    assert!(!stale.exists());
    assert!(other_tmp.exists(), "not a backup's temporary");
    assert!(hidden.exists(), "not a temporary");
    assert!(backup.exists(), "the newest backup is always kept");
    fs::remove_dir_all(dir).unwrap();
}
