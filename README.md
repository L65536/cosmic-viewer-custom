
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

[app.rs] // for [Home]/[End] = First/Last image
ViewerMessage::KeyPressed(key, modifiers, text) => {
            
if (matches!(key, Key::Named(Named::Home))|| matches!(key, Key::Named(Named::End)))
{
    let idx = if matches!(key, Key::Named(Named::Home)) {0} else {self.nav.total().saturating_sub(1)};

    if self.nav.total() > 0 {
        let thumbnail_size = self.config.thumbnail_size.pixels();
        let t = theme::active();
        let space_s = f32::from(t.cosmic().spacing.space_s);

        return Task::batch(vec![
            self.update(ViewerMessage::Nav(NavMessage::GridFocus(idx))),
            self.update(ViewerMessage::Nav(NavMessage::GridScroll(
                idx as f32 * (thumbnail_size as f32 + space_s),
            ))),
            self.update(ViewerMessage::Nav(NavMessage::GridActivate(idx))),
        ]);
    }
}            

```
[] Add OSD [current/total][folder_name/filename][image_resolution] to lower left corner.

[] Loading images from archives, e.g. ZIP // RAR CBR CBZ EPUB(ZIP)

[] New sidebar for folders/archives list view. (name/count/size/date)

[] Jump to next folder/archive when going out of bound.	
