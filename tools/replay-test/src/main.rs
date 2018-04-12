///
/// This binary will first go to the root directory (../../). Then, it will copy
/// all the files in the playback/ directory into test/replay_tests/. Then, it
/// will compile and run the zork executable, and run it for every file in
/// test/replay_tests/. It will save the output in a file called replayN.txt,
/// where N comes from the name of the playback file: playbackN.txt. (The output
/// will be fully interlaced, just as a user would see it.)
///
///

extern crate timeout_readwrite;
extern crate regex;
extern crate itertools;

#[macro_use]
extern crate log;
extern crate simplelog;

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::process::{Command, Stdio};
use std::io::{self, Read, Write};
use std::cmp;
use std::time::Duration;
use std::collections::HashMap;
use std::hash::Hash;
use timeout_readwrite::{TimeoutReader, TimeoutWriter};
use regex::Regex;
use itertools::Itertools;
use simplelog::{Config, LevelFilter, WriteLogger};

#[derive(Debug)]
struct Dirs {
   pub root_dir: PathBuf,
   pub playback_dir: PathBuf,
   pub test_dir: PathBuf,
   pub replay_test_dir: PathBuf,
   pub executable_path: PathBuf,
   pub zork_run_dir: PathBuf,
}

impl Dirs {
    pub fn new() -> Result<Dirs, io::Error> {
        // Create all the relevant paths.
        trace!("Creating paths...");
        let root_dir = Path::new("../../").canonicalize()?;
        let playback_dir = root_dir.join("playback");
        let test_dir = root_dir.join("test");
        let replay_test_dir = test_dir.join("replay_test");
        let executable_path = root_dir.join("target/debug/zork");
        let zork_run_dir = Path::new("./").canonicalize()?;

        Ok(Dirs {
            root_dir: root_dir,
            playback_dir: playback_dir,
            test_dir: test_dir,
            replay_test_dir: replay_test_dir,
            executable_path: executable_path,
            zork_run_dir: zork_run_dir,
        })
    }

    // Check if there are any playbacks in the playback directory, before we try
    // to create anything.
    pub fn playback_paths(&self) -> Result<Vec<PathBuf>, io::Error> {
        // Sanity check: Ensure the playback directory exists.
        if !self.playback_dir.exists() {
            panic!("Error: No playbacks to replay! Expected playback dir: {:?}", self.playback_dir);
        }
        
        trace!("Reading dir: {:?}", &self.playback_dir);
        let playback_paths = fs::read_dir(&self.playback_dir)?
            .map_results(|entry| entry.path())
            .try_collect()?;
        debug!("playback_paths: {:?}", &playback_paths);
        let playback_paths_filtered = playback_paths.into_iter()
            .filter(|&ref path| {
                path.file_name().unwrap()
                    .to_str().unwrap()
                    .starts_with("playback")
            })
            .collect();
        debug!("playback_paths_filtered: {:?}", &playback_paths_filtered);
        
        Ok(playback_paths_filtered)
    }

    // Create directories as necessary.
    pub fn make_missing_dirs(&self) -> Result<(), io::Error> {
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
    pub fn build_executable(&self) -> Result<(), io::Error> {
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
            let mut out_string = String::from_utf8(output.stdout).unwrap();
            out_string.push_str(&String::from_utf8(output.stderr).unwrap());
            println!("{}", &out_string);
            panic!("Failed to build executable.");
        }
        if !self.executable_path.exists() {
            panic!("Error: The executable is not at {:?}", &self.executable_path);
        }
        Ok(())
    }

    // Cleanup temporary playbacks generated while testing.
    pub fn cleanup_zork_run_dir(&self) -> Result<(), io::Error> {
        let temp_playback_dir = self.zork_run_dir.join("playbacks");
        if temp_playback_dir.exists() {
            debug!("Cleaning up temp direcory: {:?}", &temp_playback_dir);
            fs::remove_dir_all(&temp_playback_dir)?;
        }

        Ok(())
    }
}

trait IteratorTryCollect: Iterator {
    fn try_collect<T, E>(self) -> Result<Vec<T>, E>
    where Self: Iterator<Item = Result<T, E>> + Sized
    {
        let (lower, _) = self.size_hint();
        let mut ret = Vec::with_capacity(lower);
        for item in self {
            ret.push(item?);
        }
        Ok(ret)
    }
}

impl<T: Iterator + ?Sized> IteratorTryCollect for T {}

fn parse_replay_files(dirs: &Dirs) -> 
    Result<HashMap<usize, (PathBuf, PathBuf)>, io::Error>
{
    // Look for replays that already exist.
    trace!("Reading dir: {:?}", &dirs.replay_test_dir);
    let replay_paths = fs::read_dir(&dirs.replay_test_dir)?
        .map_results(|entry| entry.path())
        .try_collect()?;
    debug!("Replay paths: {:?}", &replay_paths);

    // Split the replays into the inX.txt and outX.txt files.
    let mut in_replay_paths: Vec<PathBuf> = replay_paths.iter()
        .filter(|&ref path| path.file_name().unwrap()
                .to_str().unwrap()
                .starts_with("in"))
        .map(|&ref path| path.clone())
        .collect();
    let mut out_replay_paths: Vec<PathBuf> = replay_paths.into_iter()
        .filter(|&ref path| path.file_name().unwrap()
                .to_str().unwrap()
                .starts_with("out"))
        .collect();
    debug!("in_replay_paths: {:?}", &in_replay_paths);
    debug!("out_replay_paths: {:?}", &out_replay_paths);

    in_replay_paths.sort();
    out_replay_paths.sort();
    if in_replay_paths.len() != out_replay_paths.len() {
        // Figure out why there's a discrepency, and output a meaningful error.
        find_missing_replay_file(&in_replay_paths, &out_replay_paths);
        unreachable!();
    }

    // Extract the id numbers for the paths.
    let replay_id_numbers = get_id_numbers(&in_replay_paths).unwrap();
    debug!("Extracted id numbers: {:?}", &replay_id_numbers);

    // Zip the replay files together.
    let in_iter = in_replay_paths.into_iter();
    let out_iter = out_replay_paths.into_iter();
    let replay_pairs: Vec<(PathBuf, PathBuf)> = in_iter.zip(out_iter).collect();

    // Make the final mapping using arrays.
    let ret = vecs_into_map(replay_id_numbers, replay_pairs);

    Ok(ret)
}

fn fix_playback_files(
    playback_map: &mut HashMap<usize, PathBuf>,
    replay_map: &HashMap<usize, (PathBuf, PathBuf)>,
    dirs: &Dirs
) {
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
        let playback = playback_map.get(&id).unwrap();
        let replay_in = &(*replay_map.get(&id).unwrap()).0;
        if files_are_equal(&playback, &replay_in) {
            skip_ids.push(id);
        } else {
            rename_ids.push(id);
        }
    }
    println!("Skipping {}/{} playbacks.", skip_ids.len(), playback_map.len());
    for id in skip_ids {
        let skipped_path = playback_map.remove(&id);
        info!("Skipping playback file: {:?}", &skipped_path);
    }
    let rename_ids = rename_ids;

    // Get the highest id number being used right now.
    let highest: usize = replay_map.keys().fold(0, |highest, &k| cmp::max(highest, k));

    // Rename each ID, and the corresponding playback file.
    let iter = rename_ids.into_iter().zip((highest + 1)..);
    for (old_id, new_id) in iter {
        let maybe_path = playback_map.remove(&old_id);
        let old_path = maybe_path.expect(&format!("Error renaming id {}", old_id));
        let new_path = dirs.playback_dir.join(format!("playback{}.txt", new_id));
        info!("Renaming playback file {:?} to {:?}", &old_path, &new_path);
        fs::copy(&old_path, &new_path).unwrap();
        playback_map.insert(new_id, new_path);
    }

}

fn main() {
    WriteLogger::init(
        LevelFilter::Trace,
        Config::default(),
        File::create("replay_test_log.txt").unwrap()
    ).unwrap();

    let dirs = Dirs::new().unwrap();
    debug!("dirs: {:?}", dirs);
    
    // Check if there are any playbacks in the playback directory, before we try
    // to create anything.
    let playback_paths = dirs.playback_paths().unwrap();
    if playback_paths.len() == 0 {
        panic!("Error: No playbacks to replay! None found in the dir: {:?}", dirs.playback_dir);
    }

    // Create directories as necessary.
    dirs.make_missing_dirs().unwrap();

    // Cleanup anything left from previous runs.
    dirs.cleanup_zork_run_dir().unwrap();
    
    // Figure out the ids for the replay maps.
    let replay_map = parse_replay_files(&dirs).unwrap();

    // Extract ID numbers.
    let playback_id_numbers = get_id_numbers(&playback_paths).unwrap();

    // Create a map from ID number to path name for playbacks and replays.
    let mut playback_map = vecs_into_map(playback_id_numbers, playback_paths);

    // Rename playback files as needed and filter out duplicates.
    fix_playback_files(&mut playback_map, &replay_map, &dirs);
    debug!("Playback map after fixing: {:?}", &playback_map);
    
    // Finally, make sure that we have an executable to use for testing.
    dirs.build_executable().unwrap();
    
    // Now we have our final set of playback files and their ID's. Copy them to
    // the test directory, and generate output.
    let playback_count = playback_map.len();
    for (index, (id, playback_path)) in playback_map.into_iter().enumerate() {
        
        let progress = format!("({}/{}) Running test for: {:?}", index + 1, playback_count, &playback_path);
        println!("{}", progress);
        info!("{}", progress);

        // Make the new file names.
        let dest_in_path = dirs.replay_test_dir.join(format!("in{}.txt", id));
        let dest_out_path = dirs.replay_test_dir.join(format!("out{}.txt", id));

        // Run the test to create an output file.
        run_playback_test(&playback_path, &dirs, &dest_out_path).unwrap();

        // Copy the old playback file into the test dir.
        // We do this after testing, so that it will not be counted as "complete"
        // if we are stopped in the middle of a test via Control-C.
        info!("Copying old playback to {:?}", &dest_in_path);
        fs::copy(&playback_path, &dest_in_path).unwrap();
       
    }
}

fn run_playback_test(playback_path: &PathBuf, dirs: &Dirs, dest_out_path: &PathBuf) ->
    Result<(), io::Error>
{
    // Read the playback file.
    let mut playback = String::new();
    let mut playback_file = File::open(&playback_path)?;
    playback_file.read_to_string(&mut playback)?;

    // Se need to append a "yes" to the end of the playback, to answer the "Do
    // you really want to quit?" prompt.
    playback.push_str("\nquit\nyes");
    let playback = playback;

    // Split the string into an array of lines.
    let lines: Vec<&str> = playback.split("\n").collect();

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
    let mut stdin = TimeoutWriter::new(child.stdin.take().unwrap(), Duration::new(1, 0));
    let mut stdout = TimeoutReader::new(child.stdout.take().unwrap(), Duration::new(1, 0));
    //let mut stdin = child.stdin.take().unwrap();
    //let mut stdout = child.stdout.take().unwrap();

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

    // Read anything else from the child.
    debug!("Reading anything else from child...");
    let output = read_from_child(&mut stdout)?;
    out_buffer.push_str(&output);

    // Save all the output.
    debug!("Saving output to {:?}", &dest_out_path);
    let mut out_file = File::create(&dest_out_path)?;
    write_to_file(&mut out_file, &out_buffer)?;

    // Check that the child is done.
    if let None = child.try_wait()? {
        panic!("Child process won't quit");
    }

    Ok(())
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

fn files_are_equal<P: AsRef<Path>, Q: AsRef<Path>>(a: P, b: Q) -> bool {
   Command::new("diff")
       .arg(a.as_ref().canonicalize().unwrap())
       .arg(b.as_ref().canonicalize().unwrap())
       .stdin(Stdio::null())
       .stdout(Stdio::null())
       .stderr(Stdio::null())
       .status()
       .unwrap()
       .success()
}

fn read_from_child<R: Read>(child: &mut R) -> 
    Result<String, io::Error>
{
    let mut buffer_str = String::new();
    loop {
        let mut buffer = [0; 100];
        let res = child.read(&mut buffer);
        let count = match res {
            Ok(count) => count,
            Err(err) => {
                if err.kind() == io::ErrorKind::TimedOut || err.kind() == io::ErrorKind::BrokenPipe {
                    debug!("Timed out or broken pipe ({:?}); breaking.", err.kind());
                    break;
                }
                error!("Error reading from child: {:?}", &err);
                return Err(err);
            }
        };

        if count == 0 {
            debug!("No more stdout from child.");
            break;
        }

        let converted_buffer = String::from_utf8(buffer[0..count].to_vec()).unwrap();
        buffer_str.push_str(&converted_buffer);
    }
    
    debug!("Read from child: \n{:?}", &buffer_str);
    Ok(buffer_str)
}

fn write_to_child<W: Write>(child: &mut W, input: &str) -> Result<(), io::Error> {
    child.write(input.as_bytes())?;
    Ok(())
}

fn write_to_file<W: Write>(mut file: &mut W, input: &str) -> Result<(), io::Error> {
    write_to_child(&mut file, &input)
}

fn get_id_numbers(paths: &Vec<PathBuf>) -> Result<Vec<usize>, ()> {
    let regex = Regex::new(r"(\d*).txt$").unwrap();
    let mut numbers = Vec::with_capacity(paths.len());
    for path in paths {
        let name = path.file_name().unwrap().to_string_lossy();
        let maybe_captures = regex.captures(&name);
        let captures = match maybe_captures {
            Some(captures) => captures,
            None => {
                debug!("skipping playback file without number: {:?}", &name);
                continue;
            }
        };
        let maybe_number = captures[1].parse();
        match maybe_number {
            Ok(number) => numbers.push(number),
            Err(_) => panic!("Error parsing number from capture {:?} in path {:?}", &captures[1], path)
        }
    }
    Ok(numbers)
}

fn find_missing_replay_file(in_paths: &Vec<PathBuf>, out_paths: &Vec<PathBuf>) -> !
{
    let mut in_numbers = get_id_numbers(&in_paths).unwrap();
    let mut out_numbers = get_id_numbers(&out_paths).unwrap();
    in_numbers.sort();
    out_numbers.sort();
    let iter = in_numbers.into_iter().zip(out_numbers.into_iter());
    for (in_number, out_number) in iter {
        if in_number != out_number {
            let smaller = cmp::min(in_number, out_number);
            panic!("Missing replay file pair for number {}.", smaller);
        }
    }

    unreachable!();
}

