
# Plans for the fork

[DONE] Add /.github/workflows/build.yml for Github Actions to build executables on workflow_dispatch (manual trigger).
```
Takes 10 minutes to compile online. 
2 minutes to download.
File size 69MB (original was 33MB). 
Need to try other build/compile settings.
```


[] Auto maximize window on launch

[SKIP] Auto continue from last session on launch
```
Use path in command line argument instead. 
```

[DONE] Remove toolbar in full screen mode
```
[app.rs]
.push_maybe(has_image.then(|| {    
.push_maybe((has_image && !self.is_fullscreen).then(|| {
```

[] Add/Change key/mouse bindings
```
Mouse scroll = page up/down
[+] [-] = Zoom
[F5] [mid-click] = Full screen toggle
[Home]/[End] = First/Last image
// [F11] was already used for windowed/maximize toggle.(by OS/DE?)

[key_binds.rs]
    iced::keyboard::{Key, Modifiers, key::Named},
    // iced::keyboard::{Key, Modifiers},

    // for key [+] [-]
    modifiers: vec![]  

    // [F5]
    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::F5),
        },
        MenuAction::Fullscreen,
    );
```
[] Add OSD [current/total][folder_name/filename][image_resolution] to lower left corner.

[] Loading images from archives, e.g. ZIP // RAR CBR CBZ EPUB(ZIP)

[] New sidebar for folders/archives list view. (name/count/size/date)

[] Jump to next folder/archive when going out of bound.	
