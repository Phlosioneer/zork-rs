#![feature(never_type)]

///
/// This binary will first go to the root directory (../../). Then, it will copy
/// all the files in the playback/ directory into test/replay_tests/. Then, it
/// will compile and run the zork executable, and run it for every file in
/// test/replay_tests/. It will save the output in a file called replayN.txt,
/// where N comes from the name of the playback file: playbackN.txt. (The output
/// will be fully interlaced, just as a user would see it.)
///
///
extern crate failure;
extern crate itertools;
extern crate regex;
extern crate tempdir;
extern crate timeout_readwrite;

#[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate log;

use itertools::Itertools;
use regex::Regex;
use std::cmp;
use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::Hash;
use std::io::{self, Read, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tempdir::TempDir;
use timeout_readwrite::{TimeoutReader, TimeoutWriter};

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug)]
pub struct Dirs {
    // The root directory that all the other directories are calculated from.
    // Log files are written to the root directory if a failure happens.
    pub root_dir: PathBuf,

    // The directory with playback files to use for testing.
    pub playback_dir: PathBuf,

    // The root directory for zork tests.
    // Nothing actually touches this directory.
    pub test_dir: PathBuf,

    // The directory for test output files.
    pub replay_test_dir: PathBuf,

    // The path to the actual executable.
    pub executable_path: PathBuf,

    // The path where the tests are run from.
    pub zork_run_dir: TempDir,
}

impl Dirs {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Dirs> {
        // Create all the relevant paths.
        trace!("Creating paths...");
        let root_dir = root.as_ref().canonicalize()?;
        let playback_dir = root_dir.join("playback");
        let test_dir = root_dir.join("tests");
        let replay_test_dir = test_dir.join("replay_test");
        let executable_path = root_dir.join("target/debug/zork");

        // Create a temporary dir to isolate the zork executable.
        let zork_run_dir = TempDir::new("zork_run_dir")?;

        let ret = Dirs {
            root_dir,
            playback_dir,
            test_dir,
            replay_test_dir,
            executable_path,
            zork_run_dir,
        };

        ret.make_missing_dirs()?;

        Ok(ret)
    }

    // Create a new dirs from an existing one. The new dirs is a clone, except
    // that it has a new TempDir.
    pub fn from_dirs(other: &Dirs) -> Result<Dirs> {
        Ok(Dirs {
            root_dir: other.root_dir.clone(),
            playback_dir: other.playback_dir.clone(),
            test_dir: other.test_dir.clone(),
            replay_test_dir: other.replay_test_dir.clone(),
            executable_path: other.executable_path.clone(),
            zork_run_dir: TempDir::new("zork_run_dir")?,
        })
    }

    // Check if there are any playbacks in the playback directory, before we try
    // to create anything.
    pub fn playback_paths(&self) -> Result<Vec<PathBuf>> {
        // Sanity check: Ensure the playback directory exists.
        if !self.playback_dir.exists() {
            return Err(ReplayError::MissingPlaybacks(self.playback_dir.clone()).into());
        }

        trace!("Reading dir: {:?}", &self.playback_dir);
        let playback_paths = fs::read_dir(&self.playback_dir)?
            .map_results(|entry| entry.path())
            .try_collect()?;
        debug!("playback_paths: {:?}", &playback_paths);
        let playback_paths_filtered_res: Result<Vec<PathBuf>> = playback_paths
            .into_iter()
            .try_filter(|path| {
                Ok(path.file_name()
                    .ok_or_else(|| ReplayError::NotAFile(path.clone()))?
                    .to_string_lossy()
                    .starts_with("playback"))
            })
            .try_collect();
        let playback_paths_filtered = playback_paths_filtered_res?;
        debug!("playback_paths_filtered: {:?}", &playback_paths_filtered);

        Ok(playback_paths_filtered)
    }

    // Create directories as necessary.
    fn make_missing_dirs(&self) -> Result<()> {
        if !self.test_dir.exists() {
            trace!("Creating dir: {:?}", &self.test_dir);
            fs::create_dir(&self.test_dir)?;
        }
        if !self.replay_test_dir.exists() {
            trace!("Creating dir: {:?}", &self.replay_test_dir);
            fs::create_dir(&self.replay_test_dir)?;
        }

        Ok(())
    }

    // Build the zork executable.
    pub fn build_executable(&self) -> Result<()> {
        trace!("Building executable...");
        println!("Building executable...");
        let output = Command::new("cargo")
            .arg("build")
            .current_dir(&self.root_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        if !output.status.success() {
            let stdout = String::from_utf8(output.stdout)?;
            let stderr = String::from_utf8(output.stderr)?;

            Err(ReplayError::BuildError { stdout, stderr }.into())
        } else if !self.executable_path.exists() {
            Err(ReplayError::MissingExeError(self.executable_path.clone()).into())
        } else {
            Ok(())
        }
    }

    // Get the location of the temporary log file created by zork.
    pub fn temp_log_file(&self) -> PathBuf {
        self.zork_run_dir.as_ref().join("trace_log.txt")
    }

    // Get the location of the log file of the most recently failed test.
    pub fn failing_log_file(&self) -> PathBuf {
        self.root_dir.join("test_failure_log.txt")
    }
}

pub trait IterResultTools: Iterator {
    fn try_collect<T, E>(self) -> std::result::Result<Vec<T>, E>
    where
        Self: Iterator<Item = std::result::Result<T, E>> + Sized,
    {
        let (lower, _) = self.size_hint();
        let mut ret = Vec::with_capacity(lower);
        for item in self {
            ret.push(item?);
        }
        Ok(ret)
    }

    fn try_filter<T, E, P>(self, predicate: P) -> TryFilter<Self, E, P>
    where
        P: FnMut(&T) -> std::result::Result<bool, E>,
        Self: Iterator<Item = T> + Sized,
    {
        TryFilter {
            iter: self,
            predicate,
            err_type: PhantomData,
        }
    }
}

impl<T: Iterator + ?Sized> IterResultTools for T {}

pub struct TryFilter<I, E, P> {
    iter: I,
    predicate: P,
    err_type: PhantomData<E>,
}

impl<I, E, P> Iterator for TryFilter<I, E, P>
where
    I: Iterator,
    P: FnMut(&<I as Iterator>::Item) -> std::result::Result<bool, E>,
{
    type Item = std::result::Result<<I as Iterator>::Item, E>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(value) => match (self.predicate)(&value) {
                    Ok(true) => return Some(Ok(value)),
                    Ok(false) => continue,
                    Err(error) => return Some(Err(error)),
                },
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

#[allow(unreachable_code)]
pub fn parse_replay_files(dirs: &Dirs) -> Result<HashMap<usize, (PathBuf, PathBuf)>> {
    // Look for replays that already exist.
    trace!("Reading dir: {:?}", &dirs.replay_test_dir);
    let replay_paths = fs::read_dir(&dirs.replay_test_dir)?
        .map_results(|entry| entry.path())
        .try_collect()?;
    debug!("Replay paths: {:?}", &replay_paths);

    // Split the replays into the inX.txt and outX.txt files.
    let in_replay_paths_res: Result<Vec<PathBuf>> = replay_paths
        .iter()
        .try_filter(|&path| {
            Ok(path.file_name()
                .ok_or_else(|| ReplayError::NotAFile(path.clone()))?
                .to_string_lossy()
                .starts_with("in"))
        })
        .map_results(|path| path.clone())
        .try_collect();
    let mut in_replay_paths = in_replay_paths_res?;

    let out_replay_paths_res: Result<Vec<PathBuf>> = replay_paths
        .into_iter()
        .try_filter(|path| {
            Ok(path.file_name()
                .ok_or_else(|| ReplayError::NotAFile(path.clone()))?
                .to_string_lossy()
                .starts_with("out"))
        })
        .try_collect();
    let mut out_replay_paths = out_replay_paths_res?;

    debug!("in_replay_paths: {:?}", &in_replay_paths);
    debug!("out_replay_paths: {:?}", &out_replay_paths);

    in_replay_paths.sort();
    out_replay_paths.sort();
    if in_replay_paths.len() != out_replay_paths.len() {
        // Figure out why there's a discrepency, and output a meaningful error.
        find_missing_replay_file(&in_replay_paths, &out_replay_paths)?;

        // FIXME: Figure out a better way to do this hack.
        panic!("unreachable!");
    }

    // Extract the id numbers for the paths.
    let replay_id_numbers = get_id_numbers(&in_replay_paths)?;
    debug!("Extracted id numbers: {:?}", &replay_id_numbers);

    // Zip the replay files together.
    let in_iter = in_replay_paths.into_iter();
    let out_iter = out_replay_paths.into_iter();
    let replay_pairs: Vec<(PathBuf, PathBuf)> = in_iter.zip(out_iter).collect();

    // Make the final mapping using arrays.
    let ret = vecs_into_map(replay_id_numbers, replay_pairs);

    Ok(ret)
}

pub fn parse_playback_files(dirs: &Dirs) -> Result<HashMap<usize, PathBuf>> {
    // Check if there are any playbacks in the playback directory, before we try
    // to create anything.
    let playback_paths = dirs.playback_paths()?;
    if playback_paths.len() == 0 {
        return Err(ReplayError::NoPlaybacks(dirs.playback_dir.clone()).into());
    }

    // Extract ID numbers.
    let playback_id_numbers = get_id_numbers(&playback_paths)?;

    // Create a map from ID number to path name for playbacks and replays.
    Ok(vecs_into_map(playback_id_numbers, playback_paths))
}

pub fn fix_playback_files(
    playback_map: &mut HashMap<usize, PathBuf>,
    replay_map: &HashMap<usize, (PathBuf, PathBuf)>,
    dirs: &Dirs,
) -> Result<()> {
    // Check if there are any common IDs.
    let mut overlap = Vec::<usize>::new();
    for &key in playback_map.keys() {
        if replay_map.contains_key(&key) {
            overlap.push(key);
        }
    }
    let overlap = overlap;

    // For each overlap, check if the playback file and input replay file are really
    // the same. If they are, we can skip creating new replays for them.
    let mut skip_ids: Vec<usize> = Vec::new();
    let mut rename_ids: Vec<usize> = Vec::new();
    for id in overlap {
        let playback = playback_map
            .get(&id)
            .ok_or_else(|| ReplayError::UnknownMapError(id, "playback"))?;
        let replay_paths = replay_map
            .get(&id)
            .ok_or_else(|| ReplayError::UnknownMapError(id, "replay_paths"))?;
        let replay_in = &replay_paths.0;
        if files_are_equal(playback, replay_in)? {
            skip_ids.push(id);
        } else {
            rename_ids.push(id);
        }
    }
    println!(
        "Skipping {}/{} playbacks.",
        skip_ids.len(),
        playback_map.len()
    );
    for id in skip_ids {
        let skipped_path = playback_map.remove(&id);
        info!("Skipping playback file: {:?}", &skipped_path);
    }
    let rename_ids = rename_ids;

    // Get the highest id number being used right now.
    let highest: usize = replay_map
        .keys()
        .fold(0, |highest, &k| cmp::max(highest, k));

    // Rename each ID, and the corresponding playback file.
    let iter = rename_ids.into_iter().zip((highest + 1)..);
    for (old_id, new_id) in iter {
        let old_path = playback_map
            .remove(&old_id)
            .ok_or_else(|| ReplayError::RenameError(old_id))?;
        let new_path = dirs.playback_dir.join(format!("playback{}.txt", new_id));
        info!("Renaming playback file {:?} to {:?}", &old_path, &new_path);
        fs::copy(&old_path, &new_path)?;
        playback_map.insert(new_id, new_path);
    }

    Ok(())
}

pub fn run_playback_test(playback_path: &PathBuf, dirs: &Dirs) -> Result<String> {
    // Read the playback file.
    let mut playback = String::new();
    let mut playback_file = File::open(&playback_path)?;
    playback_file.read_to_string(&mut playback)?;
    let playback = playback;

    // Split the string into an array of lines.
    let mut lines: Vec<&str> = playback.split('\n').collect();
    if lines[lines.len() - 1] == "" {
        let len = lines.len();
        lines.remove(len - 1);
    }

    // Run zork to get the output.
    let mut child = Command::new(&dirs.executable_path)
        .current_dir(&dirs.zork_run_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Get the child's stdin and stdout pipes.
    // This uses Option::take() so that it doesn't partially-move the child struct,
    // allowing it to be used later for Child::wait() and Child::kill().
    let raw_stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => return Err(ReplayError::MissingStdin(child).into()),
    };

    let raw_stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => return Err(ReplayError::MissingStdout(child).into()),
    };

    let mut stdin = TimeoutWriter::new(raw_stdin, Duration::new(1, 0));
    let mut stdout = TimeoutReader::new(raw_stdout, Duration::new(1, 0));
    //let mut stdin = child.stdin.take()?;
    //let mut stdout = child.stdout.take()?;

    // Make a buffer to store the output.
    let mut out_buffer = String::with_capacity(playback.len() * 2);

    // Wait for the child to start up (at least SOME input)
    trace!("Waiting for child to start.");
    while out_buffer.len() == 0 {
        let output = read_from_child(&mut stdout)?;
        out_buffer.push_str(&output);
    }

    // Send the input to the child, line by line.
    trace!("Starting tests.");
    let line_count = lines.len();
    for (index, line) in lines.iter().enumerate() {
        println!("Line {}/{}", index + 1, line_count);

        // Read everything from the child so far.
        let output = read_from_child(&mut stdout)?;
        out_buffer.push_str(&output);

        // Recover the newline that was used to split the input lines.
        let mut line_with_newline = line.to_string();
        line_with_newline.push_str("\n");

        // Send a line of input to the child.
        debug!("Sending line: {:?}", &line);
        out_buffer.push_str(&line_with_newline);
        write_to_child(&mut stdin, &line_with_newline)?;
    }

    // Tell the program to quit. It's okay if we get a BrokenPipe error here;
    // it just means the program quit on its own.
    let last_lines = vec!["quit\n", "yes\n"];
    for line in last_lines {
        let maybe_output = read_from_child(&mut stdout);
        match maybe_output {
            Ok(output) => out_buffer.push_str(&output),
            Err(err) => {
                let io_error = err.downcast::<io::Error>()?;
                if io_error.kind() == io::ErrorKind::BrokenPipe {
                    debug!("Child closed itself. (stdout first)");
                    break;
                } else {
                    return Err(io_error.into());
                }
            }
        }

        debug!("Sending line: {:?}", &line);
        out_buffer.push_str(&line);
        let res = write_to_child(&mut stdin, &line);
        if let Err(err) = res {
            let io_error = err.downcast::<io::Error>()?;
            if io_error.kind() == io::ErrorKind::BrokenPipe {
                debug!("Child closed itself. (stdin first)");
                break;
            } else {
                return Err(io_error.into());
            }
        }
    }

    // Read anything else from the child.
    debug!("Reading anything else from child...");
    let output = read_from_child(&mut stdout)?;
    out_buffer.push_str(&output);

    // Check that the child is done.
    if child.try_wait()? == None {
        return Err(ReplayError::TerminationError(child).into());
    }

    Ok(out_buffer)
}

fn vecs_into_map<K: Hash + Eq, V>(keys: Vec<K>, values: Vec<V>) -> HashMap<K, V> {
    assert_eq!(keys.len(), values.len());
    let len = keys.len();
    let iter = keys.into_iter().zip(values.into_iter());
    let mut map = HashMap::with_capacity(len);
    for (key, value) in iter {
        map.insert(key, value);
    }
    map
}

pub fn files_are_equal(a: &PathBuf, b: &PathBuf) -> Result<bool> {
    Ok(Command::new("diff")
        .arg(<PathBuf as AsRef<Path>>::as_ref(&a).canonicalize()?)
        .arg(<PathBuf as AsRef<Path>>::as_ref(&b).canonicalize()?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success())
}

fn read_from_child<R: Read>(child: &mut R) -> Result<String> {
    let mut buffer_str = String::new();
    loop {
        let mut buffer = [0; 100];
        let res = child.read(&mut buffer);
        let count = match res {
            Ok(count) => count,
            Err(err) => {
                if err.kind() == io::ErrorKind::TimedOut || err.kind() == io::ErrorKind::BrokenPipe
                {
                    debug!("Timed out or broken pipe ({:?}); breaking.", err.kind());
                    break;
                }
                error!("Error reading from child: {:?}", &err);
                return Err(err.into());
            }
        };

        if count == 0 {
            debug!("No more stdout from child.");
            break;
        }

        let converted_buffer = String::from_utf8(buffer[0..count].to_vec())?;
        buffer_str.push_str(&converted_buffer);

        if buffer_str.ends_with("\n>") {
            debug!("Prompt found.");
            break;
        }
    }

    debug!("Read from child: \n{:?}", &buffer_str);
    Ok(buffer_str)
}

fn write_to_child<W: Write>(child: &mut W, input: &str) -> Result<()> {
    child.write_all(input.as_bytes())?;
    Ok(())
}

fn get_id_numbers(paths: &Vec<PathBuf>) -> Result<Vec<usize>> {
    let regex = Regex::new(r"(\d*).txt$")?;
    let mut numbers = Vec::with_capacity(paths.len());
    for path in paths {
        let name = path.file_name()
            .ok_or_else(|| ReplayError::NotAFile(path.clone()))?
            .to_string_lossy();
        let maybe_captures = regex.captures(&name);
        let captures = match maybe_captures {
            Some(captures) => captures,
            None => {
                debug!("skipping playback file without number: {:?}", &name);
                continue;
            }
        };
        let number = captures[1].parse()?;
        numbers.push(number);
    }
    Ok(numbers)
}

fn find_missing_replay_file(in_paths: &Vec<PathBuf>, out_paths: &Vec<PathBuf>) -> Result<!> {
    let mut in_numbers = get_id_numbers(&in_paths)?;
    let mut out_numbers = get_id_numbers(&out_paths)?;
    in_numbers.sort();
    out_numbers.sort();
    let iter = in_numbers.into_iter().zip(out_numbers.into_iter());
    for (in_number, out_number) in iter {
        if in_number != out_number {
            let smaller = cmp::min(in_number, out_number);
            return Err(ReplayError::MissingFilePairError(smaller).into());
        }
    }

    unreachable!()
}

#[derive(Debug, Fail)]
pub enum ReplayError {
    #[fail(display = "No playbacks to replay! Expected playback dir: {:?}", _0)]
    MissingPlaybacks(PathBuf),

    #[fail(
        display = "
Error while building zork.
Stdout:
----------
{}
----------
Stderr:
{}
----------
",
        stdout,
        stderr
    )]
    BuildError { stdout: String, stderr: String },

    #[fail(display = "Error: The executable is not at {:?}", _0)]
    MissingExeError(PathBuf),

    #[fail(display = "Child process won't quit")]
    TerminationError(Child),

    #[fail(display = "Missing replay file pair for number {}.", _0)]
    MissingFilePairError(usize),

    #[fail(display = "Path {:?} is not a file.", _0)]
    NotAFile(PathBuf),

    #[fail(display = "Failed to create stdout pipe for child.")]
    MissingStdout(Child),

    #[fail(display = "Failed to create stdin pipe for child.")]
    MissingStdin(Child),

    #[fail(display = "Error renaming playback id {}.", _0)]
    RenameError(usize),

    #[fail(display = "Map \"{}\" has no key {}.", _0, _1)]
    UnknownMapError(usize, &'static str),

    #[fail(display = "Error: No playbacks to replay! None found in the dir: {:?}", _0)]
    NoPlaybacks(PathBuf),
}
