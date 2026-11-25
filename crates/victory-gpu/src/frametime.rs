//! The frame time dump (`VICTORY_FRAMETIME_DUMP`, diagnostics builds): a CSV
//! row per one-second frame summary with the figures the summary already
//! measures (preparing, copies, textures, the GPU, copying back, draws,
//! dropped work), and the worst and mean single frame of that second, timed
//! at each present. The file starts with `#` lines naming the machine (CPU,
//! GPU and its video memory, driver, RAM, OS, display), the `VICTORY_*`
//! switches and the title build, so dumps from two PCs can be compared.
//! They name no user, path or serial: a switch whose value is a path is
//! listed without it.
//!
//! Ported from skornuta's frame time dump (skornuta/victory-fork, packaged
//! as skornuta/victory-frametime, GPL-3.0), cut down to the figures this
//! summary has.

use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Where the dump goes for a `VICTORY_FRAMETIME_DUMP` value: `1` is
/// `frametime.csv` in the cache folder, unset, empty or `0` is none, and
/// anything else is that file.
pub fn dump_path(value: Option<&OsStr>, cache_dir: &Path) -> Option<PathBuf> {
    match value {
        None => None,
        Some(v) if v.is_empty() || v == "0" => None,
        Some(v) if v == "1" => Some(cache_dir.join("frametime.csv")),
        Some(v) => Some(PathBuf::from(v)),
    }
}

enum State {
    /// Named by the app; opened with the first row, when the host device
    /// the header names exists.
    Waiting { path: PathBuf, build: String },
    Open { file: std::fs::File, start: Instant },
    /// The file could not be made (logged once).
    Failed,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

/// Writes the dump to `path` (truncated), naming the title build `build`
/// in its header. Called by the app before the title runs.
pub fn start(path: PathBuf, build: String) {
    *STATE.lock().unwrap_or_else(|e| e.into_inner()) = Some(State::Waiting { path, build });
}

/// The time between presents over one summary interval.
#[derive(Default)]
pub(crate) struct Frames {
    last: Option<Instant>,
    worst: Duration,
    total: Duration,
    timed: u32,
}

impl Frames {
    /// A frame was presented at `now`.
    pub(crate) fn presented(&mut self, now: Instant) {
        if let Some(last) = self.last {
            let frame = now.saturating_duration_since(last);
            self.worst = self.worst.max(frame);
            self.total += frame;
            self.timed += 1;
        }
        self.last = Some(now);
    }

    /// The worst and the mean frame since the last call, in ms (0 when no
    /// frame was timed), and starts the next interval.
    pub(crate) fn take(&mut self) -> (f64, f64) {
        let worst = self.worst.as_secs_f64() * 1000.0;
        let mean = match self.timed {
            0 => 0.0,
            n => self.total.as_secs_f64() * 1000.0 / f64::from(n),
        };
        (self.worst, self.total, self.timed) = (Duration::ZERO, Duration::ZERO, 0);
        (worst, mean)
    }
}

const COLUMNS: &str = "t_s,frame,frames,fps,ms_per_frame,frame_ms_max,frame_ms_avg,\
preparing_ms,copies_ms,copies_mb,textures_ms,textures_mb,gpu_ms,copyback_ms,copyback_mb,\
draws,dropped,making,indirect,root_view_slots";

/// One summary interval, as the frame summary computes it: times in ms
/// and sizes in MB per frame, counts per frame.
pub(crate) struct Row {
    /// Frames presented so far.
    pub(crate) frame: u64,
    /// Frames in the interval.
    pub(crate) frames: f64,
    pub(crate) fps: f64,
    pub(crate) ms: f64,
    pub(crate) worst_ms: f64,
    pub(crate) mean_ms: f64,
    pub(crate) preparing_ms: f64,
    pub(crate) copies_ms: f64,
    pub(crate) copies_mb: f64,
    pub(crate) textures_ms: f64,
    pub(crate) textures_mb: f64,
    pub(crate) gpu_ms: f64,
    pub(crate) copy_back_ms: f64,
    pub(crate) copy_back_mb: f64,
    pub(crate) draws: f64,
    pub(crate) dropped: f64,
    /// Host pipelines being made at the end of the interval.
    pub(crate) making: u64,
    pub(crate) indirect: f64,
    pub(crate) root_views: f64,
}

impl Row {
    /// The CSV line, `t_s` seconds after the dump started.
    fn line(&self, t_s: f64) -> String {
        let r = self;
        format!(
            "{t_s:.3},{},{:.0},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},\
             {:.1},{:.1},{},{:.1},{:.1}",
            r.frame,
            r.frames,
            r.fps,
            r.ms,
            r.worst_ms,
            r.mean_ms,
            r.preparing_ms,
            r.copies_ms,
            r.copies_mb,
            r.textures_ms,
            r.textures_mb,
            r.gpu_ms,
            r.copy_back_ms,
            r.copy_back_mb,
            r.draws,
            r.dropped,
            r.making,
            r.indirect,
            r.root_views,
        )
    }
}

/// Appends `row` when the dump is on; the first row makes the file.
pub(crate) fn record(row: &Row) {
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(State::Waiting { path, build }) = state.take_if(|s| matches!(s, State::Waiting { .. })) {
        *state = Some(match open(&path, &build) {
            Ok(file) => {
                tracing::info!(path = %path.display(), "frame time dump");
                State::Open { file, start: Instant::now() }
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "no frame time dump");
                State::Failed
            }
        });
    }
    let Some(State::Open { file, start }) = state.as_mut() else { return };
    // Flushed per row (one a second), so a crash keeps what came before.
    let _ = writeln!(file, "{}", row.line(start.elapsed().as_secs_f64()));
    let _ = file.flush();
}

fn open(path: &Path, build: &str) -> std::io::Result<std::fs::File> {
    let mut file = std::fs::File::create(path)?;
    let switches = switches(std::env::vars_os().filter_map(|(k, v)| {
        Some((k.into_string().ok()?, v.to_string_lossy().into_owned()))
    }));
    write!(file, "{}", header(build, &machine(), &switches))?;
    writeln!(file, "{COLUMNS}")?;
    file.flush()?;
    Ok(file)
}

/// The `#` lines before the columns (`pandas.read_csv(..., comment='#')`
/// skips them).
fn header(build: &str, machine: &[(&str, String)], switches: &str) -> String {
    let mut out = format!(
        "# victory frame time dump\n# victory: {}\n# title: {build}\n",
        env!("CARGO_PKG_VERSION")
    );
    for (name, value) in machine {
        out += &format!("# {name}: {}\n", one_line(value));
    }
    if !switches.is_empty() {
        out += &format!("# switches: {switches}\n");
    }
    out
}

fn one_line(s: &str) -> String {
    s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect()
}

/// The `VICTORY_*` variables that are set, sorted; a value that is a path
/// (it has a slash) is shown as `<path>`, as it may name the user.
fn switches(vars: impl Iterator<Item = (String, String)>) -> String {
    let mut set: Vec<String> = vars
        .filter(|(key, value)| key.starts_with("VICTORY_") && !value.is_empty())
        .map(|(key, value)| {
            if value.contains(['/', '\\']) {
                format!("{key}=<path>")
            } else {
                format!("{key}={}", one_line(&value))
            }
        })
        .collect();
    set.sort();
    set.join(" ")
}

/// NVIDIA's driver number as people quote it, from the Windows driver
/// version: 32.0.15.6094 is 560.94 (the last digit of the third part and
/// the fourth).
fn nvidia_driver(version: [u16; 4]) -> String {
    let n = u32::from(version[2] % 10) * 10_000 + u32::from(version[3]);
    format!("{}.{:02}", n / 100, n % 100)
}

/// The machine, as `(name, value)` lines; what cannot be read is left out.
fn machine() -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    let logical = std::thread::available_parallelism().map_or(0, |n| n.get());
    let cpu = registry_text(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0", "ProcessorNameString")
        .map_or_else(|| "unknown".to_owned(), |name| name.trim().to_owned());
    out.push(("cpu", format!("{cpu} ({logical} logical processors)")));
    if let Some(adapter) = adapter() {
        out.push(("gpu", format!("{} ({} MiB dedicated)", adapter.name, adapter.video_memory >> 20)));
        if let Some(v) = adapter.driver {
            let windows = format!("{}.{}.{}.{}", v[0], v[1], v[2], v[3]);
            out.push((
                "driver",
                if adapter.vendor == 0x10de { format!("{} ({windows})", nvidia_driver(v)) } else { windows },
            ));
        }
    }
    if let Some(mib) = total_ram_mib() {
        out.push(("ram", format!("{mib} MiB")));
    }
    let os = |value: &str| registry_text(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", value);
    let build = os("CurrentBuild");
    let mut product = os("ProductName").unwrap_or_default();
    // Windows 11 still says "Windows 10" in `ProductName`.
    if build.as_deref().and_then(|b| b.parse::<u32>().ok()).is_some_and(|b| b >= 22_000) {
        product = product.replace("Windows 10", "Windows 11");
    }
    if let Some(release) = os("DisplayVersion") {
        product += &format!(" {release}");
    }
    if let Some(build) = build {
        product += &format!(" (build {build})");
    }
    if !product.trim().is_empty() {
        out.push(("os", product.trim().to_owned()));
    }
    if let Some(wine) = wine_version() {
        out.push(("wine", wine));
    }
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    // SAFETY: plain queries of the primary display's size.
    let (w, h) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    if w > 0 && h > 0 {
        out.push(("display", format!("{w}x{h}")));
    }
    out
}

struct Adapter {
    name: String,
    vendor: u32,
    video_memory: u64,
    /// The user-mode driver's version, when DXGI reports it.
    driver: Option<[u16; 4]>,
}

/// The adapter the host device runs on.
fn adapter() -> Option<Adapter> {
    use windows::Win32::Graphics::Dxgi::*;
    use windows::core::Interface;
    let host = crate::host::host()?;
    // SAFETY: plain DXGI queries on the adapter the host device was made on.
    unsafe {
        let factory: IDXGIFactory4 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0)).ok()?;
        let adapter: IDXGIAdapter1 = factory.EnumAdapterByLuid(host.device.GetAdapterLuid()).ok()?;
        let desc = adapter.GetDesc1().ok()?;
        let end = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
        let driver = adapter.CheckInterfaceSupport(&IDXGIDevice::IID).ok().map(|v| {
            let v = v as u64;
            [(v >> 48) as u16, (v >> 32) as u16, (v >> 16) as u16, v as u16]
        });
        Some(Adapter {
            name: String::from_utf16_lossy(&desc.Description[..end]),
            vendor: desc.VendorId,
            video_memory: desc.DedicatedVideoMemory as u64,
            driver,
        })
    }
}

/// A text value under `HKEY_LOCAL_MACHINE`.
fn registry_text(key: &str, name: &str) -> Option<String> {
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW};
    let wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };
    let (key, name) = (wide(key), wide(name));
    let mut buffer = vec![0u16; 256];
    let mut size = (buffer.len() * 2) as u32;
    // SAFETY: `key` and `name` are NUL-terminated UTF-16 strings that outlive
    // the call, and `buffer` has room for the `size` bytes passed.
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    // shortcut: a value longer than the buffer is left out.
    let len = (size as usize / 2).min(buffer.len());
    let end = buffer[..len].iter().position(|&c| c == 0).unwrap_or(len);
    Some(String::from_utf16_lossy(&buffer[..end]))
}

/// Physical memory, in MiB.
fn total_ram_mib() -> Option<u64> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    // SAFETY: plain data, filled in by the call.
    let mut m: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    m.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
    // SAFETY: a structure of the size it says.
    let ok = unsafe { GlobalMemoryStatusEx(&mut m) } != 0;
    ok.then_some(m.ullTotalPhys >> 20)
}

/// Wine's version when running under Wine or Proton (its `ntdll` exports
/// `wine_get_version`).
fn wine_version() -> Option<String> {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    let ntdll: Vec<u16> = "ntdll.dll".encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: a NUL-terminated module name and export name; ntdll is always
    // loaded, and `wine_get_version` takes nothing and returns a static
    // NUL-terminated string.
    unsafe {
        let module = GetModuleHandleW(ntdll.as_ptr());
        if module.is_null() {
            return None;
        }
        let get = GetProcAddress(module, c"wine_get_version".as_ptr().cast())?;
        let get = std::mem::transmute::<
            unsafe extern "system" fn() -> isize,
            extern "C" fn() -> *const std::ffi::c_char,
        >(get);
        let text = get();
        (!text.is_null()).then(|| std::ffi::CStr::from_ptr(text).to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_is_the_cache_folder_and_zero_is_off() {
        let cache = Path::new("cache");
        assert_eq!(dump_path(None, cache), None);
        assert_eq!(dump_path(Some(OsStr::new("")), cache), None);
        assert_eq!(dump_path(Some(OsStr::new("0")), cache), None);
        assert_eq!(dump_path(Some(OsStr::new("1")), cache), Some(cache.join("frametime.csv")));
        assert_eq!(dump_path(Some(OsStr::new("x.csv")), cache), Some(PathBuf::from("x.csv")));
    }

    #[test]
    fn frames_are_timed_between_presents() {
        let mut frames = Frames::default();
        let t = Instant::now();
        frames.presented(t);
        assert_eq!(frames.take(), (0.0, 0.0));
        frames.presented(t + Duration::from_millis(10));
        frames.presented(t + Duration::from_millis(40));
        let (worst, mean) = frames.take();
        assert!((worst - 30.0).abs() < 1e-6);
        assert!((mean - 20.0).abs() < 1e-6);
        // The next interval starts from the last present.
        frames.presented(t + Duration::from_millis(45));
        let (worst, mean) = frames.take();
        assert!((worst - 5.0).abs() < 1e-6);
        assert!((mean - 5.0).abs() < 1e-6);
    }

    #[test]
    fn a_row_has_a_value_per_column() {
        let row = Row {
            frame: 600,
            frames: 60.0,
            fps: 59.9,
            ms: 16.7,
            worst_ms: 33.4,
            mean_ms: 16.7,
            preparing_ms: 4.0,
            copies_ms: 1.0,
            copies_mb: 2.0,
            textures_ms: 0.5,
            textures_mb: 3.0,
            gpu_ms: 9.0,
            copy_back_ms: 0.2,
            copy_back_mb: 0.1,
            draws: 1500.0,
            dropped: 2.0,
            making: 3,
            indirect: 4.0,
            root_views: 5.0,
        };
        let line = row.line(1.5);
        assert_eq!(line.split(',').count(), COLUMNS.split(',').count());
        assert!(line.starts_with("1.500,600,60,59.900,16.700,33.400,"));
    }

    #[test]
    fn switches_leave_out_paths() {
        let vars = [
            ("VICTORY_LOG", "info"),
            ("VICTORY_FRAMETIME_DUMP", r"C:\Users\someone\ft.csv"),
            ("VICTORY_CAPTURE", "/home/someone/frames"),
            ("VICTORY_EMPTY", ""),
            ("PATH", "x"),
            ("VICTORY_KEYS", "25:Enter"),
        ]
        .map(|(k, v)| (k.to_owned(), v.to_owned()));
        assert_eq!(
            switches(vars.into_iter()),
            "VICTORY_CAPTURE=<path> VICTORY_FRAMETIME_DUMP=<path> VICTORY_KEYS=25:Enter VICTORY_LOG=info"
        );
    }

    #[test]
    fn header_lines_are_comments() {
        let machine = [("gpu", "A\nB".to_owned())];
        let header = header("build (hash)", &machine, "VICTORY_LOG=info");
        assert!(header.lines().all(|l| l.starts_with("# ")));
        assert!(header.contains("# gpu: A B\n"));
        assert!(header.contains("# title: build (hash)\n"));
    }

    #[test]
    fn nvidia_numbers_come_from_the_last_five_digits() {
        assert_eq!(nvidia_driver([32, 0, 15, 6094]), "560.94");
        assert_eq!(nvidia_driver([31, 0, 15, 123]), "501.23");
    }
}
