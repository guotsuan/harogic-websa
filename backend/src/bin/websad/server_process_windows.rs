use std::{
    io::{self, Read, Write},
    mem::size_of,
    os::windows::{
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::CommandExt,
    },
    path::PathBuf,
    process::{Child, ChildStderr, ChildStdout, Command, Stdio},
    ptr,
};
use windows_sys::Win32::{
    Foundation::{ERROR_BROKEN_PIPE, ERROR_PIPE_NOT_CONNECTED},
    System::{
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
        Pipes::PeekNamedPipe,
        Threading::CREATE_NO_WINDOW,
    },
};

pub(super) fn read_pipe(
    reader: &mut (impl Read + AsRawHandle),
    buffer: &mut [u8],
) -> io::Result<usize> {
    let mut available = 0;
    let ok = unsafe {
        PeekNamedPipe(
            reader.as_raw_handle(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            &mut available,
            ptr::null_mut(),
        )
    };
    if ok == 0 {
        let error = io::Error::last_os_error();
        return match error.raw_os_error().map(|code| code as u32) {
            Some(ERROR_BROKEN_PIPE | ERROR_PIPE_NOT_CONNECTED) => Ok(0),
            _ => Err(error),
        };
    }
    if available == 0 {
        return Err(io::ErrorKind::WouldBlock.into());
    }
    let amount = buffer.len().min(available as usize);
    reader.read(&mut buffer[..amount])
}

fn application_dir() -> Option<PathBuf> {
    let mut exe = std::env::current_exe().ok()?;
    exe.pop();
    Some(exe)
}

pub(super) fn binary(name: &str) -> PathBuf {
    application_dir()
        .unwrap_or_default()
        .join("soapy")
        .join("bin")
        .join(format!("{name}.exe"))
}

pub(super) fn configure_command(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
    let sdk = std::env::var_os("WEBSA_RUST_IQS_SHIM")
        .filter(|path| !path.is_empty())
        .and_then(|path| std::path::absolute(path).ok())
        .and_then(|mut path| path.pop().then_some(path))
        .or_else(|| application_dir().map(|app| app.join("htra_api")));
    if let Some(sdk) = sdk.filter(|path| path.is_dir()) {
        // Match the SDK working directory and DLL search used by the standalone
        // server, including its device-specific CalFile directory.
        command.current_dir(&sdk);
        let mut paths = vec![sdk];
        if let Some(path) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&path));
        }
        if let Ok(path) = std::env::join_paths(paths) {
            command.env("PATH", path);
        }
    }
}

fn kill_on_close_job() -> io::Result<OwnedHandle> {
    let job = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
    if job.is_null() {
        return Err(io::Error::last_os_error());
    }
    let job = unsafe { OwnedHandle::from_raw_handle(job) };
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    let ok = unsafe {
        SetInformationJobObject(
            job.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(job)
}

pub(super) fn spawn(
    mut command: Command,
) -> io::Result<(Child, ChildStdout, ChildStderr, OwnedHandle)> {
    let job = kill_on_close_job()?;
    // The server also observes stdin EOF, including if WebSA exits between
    // process creation and job assignment.
    command
        .env("WEBSA_SOAPY_MANAGED", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut process = command.spawn()?;
    if unsafe { AssignProcessToJobObject(job.as_raw_handle(), process.as_raw_handle()) } == 0 {
        let error = io::Error::last_os_error();
        let _ = process.kill();
        let _ = process.wait();
        return Err(io::Error::other(format!(
            "cannot assign SoapySDRServer to its job: {error}"
        )));
    }
    // The packaged server waits for this byte before binding its socket.
    if let Err(error) = process.stdin.as_mut().expect("piped stdin").write_all(&[1]) {
        let _ = process.kill();
        let _ = process.wait();
        return Err(io::Error::other(format!(
            "cannot authorize SoapySDRServer startup: {error}"
        )));
    }
    let stdout = process.stdout.take().expect("piped stdout");
    let stderr = process.stderr.take().expect("piped stderr");
    Ok((process, stdout, stderr, job))
}
