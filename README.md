Disclaimer: Almost all of the committed codes in this fork are AI generated. 

# Changes and plans for the fork

### [New] Added OSD display 
- [current/total][folder_name/filename][image_resolution] in the lower left corner.
- Clicking on the OSD brings out file selection dialog.

### [New] Support browsing images inside ZIP archives
- Images are decoded in memory without extraction to disk.
- After delete, opens the next visible ZIP file in the same folder.
- Thumbnails of every images are currently disabled in this mode.

### [New] Sidebar for list view of archives. 
- When viewing images inside an archive, the sidebar lists other archives files in the same folder.
- Clickable.
- Scrollable.
- [WIP] Thumbnails of covers.

### [New Workflow] Added /.github/workflows/build.yml 
- For Github Actions to build executables on workflow_dispatch (manual trigger).
- Latest executable can be downloaded from (>)Actions tab at the top of this repository.

### [Changes] Removed toolbar in full screen mode

### [Changes] Added/changed key/mouse bindings
- [PgUp][PgDn] = Page up/down
- [+] [-] = Zoom
- [F5] [middle-click] = Full screen toggle
- [Home] [End] = First or Last page
- // [F11] was already used for windowed/maximized toggle.(by OS/DE)

# Work in progress:

- [DONE] Fix frame pacing, skipping problem
- [] Jump to next folder/archive when going out of bound.	
- [] For the listview of zip files, remove headers, change layout to single column with filename+(1 space)+filesize(interger)+(no space)+"MB"
- [] rar cbr cbz support
- [] log filename of deleted file to ~/recycled_by_image_viewer.log
- [] keyboard 1-9 enable jumping to 10% to 90% position in curent folder/zip.
- [] auto show left or right sidebar when mouse hover wihtin 0-10% or 90%-100% of app region, hide sidebars otherwise unless it was manually turned on.
- [] add "show source folder" to right mouse click menu
- [] when there is no image/archive loaded, display a text summary/list of supported files types and keyboard/mouse shorcut guide on screen, within a boxed background in a slightly different shade color, any click opens file selection diaglog
- [] For the archive file mode, add a column for Thumbnail in the listview, thumbnail using the 1st(natural sorted) image file inside each archive, preload thumnail only in the visible range of the listview.

- [SKIP] Auto maximize window on launch

- [SKIP] Auto continue from last session on launch
Use path in command line argument instead. 

Disclaimer: Almost all of the committed the codes in this fork are AI generated. 
