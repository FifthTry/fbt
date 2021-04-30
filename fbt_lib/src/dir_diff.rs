// Source: https://github.com/assert-rs/dir-diff (Apache/MIT)
// Need to modify it so including it, will send PR and try to get it included
// upstream.

use std::cmp::Ordering;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use walkdir::{DirEntry, WalkDir};

/// The various errors that can happen when diffing two directories
#[derive(Debug)]
pub enum DirDiffError {
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
    WalkDir(walkdir::Error),
}

#[derive(Debug)]
pub enum DirDiff {
    ExpectedFileMissing {
        expected: String,
    },
    ExpectedFolderMissing {
        expected: String,
    },
    UnexpectedFileFound {
        found: String,
    },
    UnexpectedFolderFound {
        found: String,
    },
    FileTypeMismatch {
        file: String,
        expected: String,
        found: String,
    },
    ContentMismatch {
        file: String,
        expected: String,
        found: String,
    },
}

pub fn diff<A: AsRef<Path>, B: AsRef<Path>>(
    a_base: A,
    b_base: B,
) -> Result<Vec<DirDiff>, DirDiffError> {
    let mut a_walker = walk_dir(a_base)?;
    let mut b_walker = walk_dir(b_base)?;

    let mut diff = vec![];

    loop {
        match (a_walker.next(), b_walker.next()) {
            (Some(a), Some(b)) => {
                // first lets check the depth:
                // a > b: UnexpectedFileFound or UnexpectedFolderFound else
                // b > a: ExpectedFileMissing or ExpectedFolderMissing

                // if file names dont match how to find if we got a new entry
                // on left or extra entry on right? how do people actually
                // calculate diff?

                // then check file type

                // finally check file content if its a file
                todo!()
            }
            (None, Some(b)) => {
                // we have something in b, but a is done, lets iterate over all
                // entries in b, and put them in UnexpectedFileFound and
                // UnexpectedFolderFound
                todo!()
            }
            (Some(a), None) => {
                // we have something in a, but b is done, lets iterate over all
                // entries in a, and put them in ExpectedFileMissing and
                // ExpectedFolderMissing
                todo!()
            }
            (None, None) => break,
        }
    }

    // for (a, b) in (&mut a_walker).zip(&mut b_walker) {
    //     let a = a?;
    //     let b = b?;
    //
    //     a_walker.next();
    //
    //     if a.depth() != b.depth()
    //         || a.file_type() != b.file_type()
    //         || a.file_name() != b.file_name()
    //         || (a.file_type().is_file() && read_to_vec(a.path())? != read_to_vec(b.path())?)
    //     {
    //         // return Ok(true);
    //     }
    // }

    Ok(diff)
}

fn walk_dir<P: AsRef<Path>>(path: P) -> Result<walkdir::IntoIter, std::io::Error> {
    let mut walkdir = WalkDir::new(path).sort_by(compare_by_file_name).into_iter();
    if let Some(Err(e)) = walkdir.next() {
        Err(e.into())
    } else {
        Ok(walkdir)
    }
}

fn compare_by_file_name(a: &DirEntry, b: &DirEntry) -> Ordering {
    a.file_name().cmp(&b.file_name())
}

fn read_to_vec<P: AsRef<Path>>(file: P) -> Result<Vec<u8>, std::io::Error> {
    let mut data = Vec::new();
    let mut file = File::open(file.as_ref())?;

    file.read_to_end(&mut data)?;

    Ok(data)
}

impl From<std::io::Error> for DirDiffError {
    fn from(e: std::io::Error) -> DirDiffError {
        DirDiffError::Io(e)
    }
}

impl From<std::path::StripPrefixError> for DirDiffError {
    fn from(e: std::path::StripPrefixError) -> DirDiffError {
        DirDiffError::StripPrefix(e)
    }
}

impl From<walkdir::Error> for DirDiffError {
    fn from(e: walkdir::Error) -> DirDiffError {
        DirDiffError::WalkDir(e)
    }
}
