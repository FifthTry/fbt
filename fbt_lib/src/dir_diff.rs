// Source: https://github.com/assert-rs/dir-diff (Apache/MIT)
// Need to modify it so including it, will send PR and try to get it included
// upstream.

/// The various errors that can happen when diffing two directories
#[derive(Debug)]
pub(crate) enum DirDiffError {
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
    WalkDir(walkdir::Error),
}

#[derive(Debug)]
pub enum DirDiff {
    ExpectedFileMissing {
        expected: std::path::PathBuf,
    },
    ExpectedFolderMissing {
        expected: std::path::PathBuf,
    },
    UnexpectedFileFound {
        found: std::path::PathBuf,
    },
    UnexpectedFolderFound {
        found: std::path::PathBuf,
    },
    FileTypeMismatch {
        file: std::path::PathBuf,
        expected: String,
        found: String,
    },
    ContentMismatch {
        file: std::path::PathBuf,
        expected: String,
        found: String,
    },
}

pub(crate) fn diff<A: AsRef<std::path::Path>, B: AsRef<std::path::Path>>(
    _a_base: A,
    _b_base: B,
) -> Result<Option<DirDiff>, DirDiffError> {
    todo!("not yet implemented")

    // let mut a_walker = walk_dir(a_base)?;
    // let mut b_walker = walk_dir(b_base)?;
    //
    // loop {
    //     match (a_walker.next(), b_walker.next()) {
    //         (Some(a), Some(b)) => {
    //             // first lets check the depth:
    //             // a > b: UnexpectedFileFound or UnexpectedFolderFound else
    //             // b > a: ExpectedFileMissing or ExpectedFolderMissing
    //
    //             // if file names dont match how to find if we got a new entry
    //             // on left or extra entry on right? how do people actually
    //             // calculate diff?
    //
    //             // then check file type
    //
    //             // finally check file content if its a file
    //
    //             // TODO: this is dummy code to test stuff
    //             let a = a?;
    //             let b = b?;
    //             eprintln!("a: {:?}", a);
    //             eprintln!("b: {:?}", b);
    //
    //             // TODO: this is fully buggy! loop never loops.
    //             let found: std::path::PathBuf = b.path().into();
    //             return Ok(Some(if found.is_dir() {
    //                 DirDiff::UnexpectedFolderFound { found }
    //             } else {
    //                 DirDiff::UnexpectedFileFound { found }
    //             }));
    //         }
    //         (None, Some(b)) => {
    //             // we have something in b, but a is done, lets iterate over all
    //             // entries in b, and put them in UnexpectedFileFound and
    //             // UnexpectedFolderFound
    //             let found: std::path::PathBuf = b?.path().into();
    //             return Ok(Some(if found.is_dir() {
    //                 DirDiff::UnexpectedFolderFound { found }
    //             } else {
    //                 DirDiff::UnexpectedFileFound { found }
    //             }));
    //         }
    //         (Some(a), None) => {
    //             // we have something in a, but b is done, lets iterate over all
    //             // entries in a, and put them in ExpectedFileMissing and
    //             // ExpectedFolderMissing
    //             let expected: std::path::PathBuf = a?.path().into();
    //             return Ok(Some(if expected.is_dir() {
    //                 DirDiff::ExpectedFolderMissing { expected }
    //             } else {
    //                 DirDiff::ExpectedFileMissing { expected }
    //             }));
    //         }
    //         (None, None) => break,
    //     }
    // }
    //
    // Ok(None)
}

// fn walk_dir<P: AsRef<std::path::Path>>(path: P) -> Result<walkdir::IntoIter, std::io::Error> {
//     let mut walkdir = walkdir::WalkDir::new(path)
//         .sort_by(compare_by_file_name)
//         .into_iter();
//     if let Some(Err(e)) = walkdir.next() {
//         Err(e.into())
//     } else {
//         Ok(walkdir)
//     }
// }

// fn compare_by_file_name(a: &walkdir::DirEntry, b: &walkdir::DirEntry) -> std::cmp::Ordering {
//     a.file_name().cmp(&b.file_name())
// }

// fn read_to_vec<P: AsRef<Path>>(file: P) -> Result<Vec<u8>, std::io::Error> {
//     use std::fs::File;
//     use std::io::Read;
//
//     let mut data = Vec::new();
//     let mut file = File::open(file.as_ref())?;
//
//     file.read_to_end(&mut data)?;
//
//     Ok(data)
// }

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
