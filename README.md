# fbt
Folder Based Test-Runner

- [ ] split into fbt and fbt_lib crates (use workspace)
-> What I observed is that to use workspace, fbt and fbt_lib should lie on the same hierarchy.
-> My outer crate name is fbt. If we want to rename that, then I think workspace will be beneficial.
-> Am I doing it right ?

- [ ] split ui from logic
- [ ] better output: use colored
- [ ] copy input folder to a new tmp folder, and run commands there
- [ ] compare with output folder if it exists
