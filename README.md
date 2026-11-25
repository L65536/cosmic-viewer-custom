Disclaimer: Almost all of the committed codes in this fork are AI generated. 

# Changes and plans for the fork

## [New function] Added OSD display 
[current/total][folder_name/filename][image_resolution] to lower left corner.

Clicking on the OSD brings out file selection dialog.

## [New function] Support browsing images inside ZIP archives
Images are decoded in memory without extraction to disk.

Thumbnails are disabled in this mode.

ZIP archives can be moved to the trash with **Delete** or **File → Move to trash**.
Use **Undo** in the confirmation toast to restore the archive. After a successful
delete, the viewer opens the next visible ZIP file in the same folder, ordered by
filename.

## [New function] Sidebar for list view of archives. 
When viewing a ZIP, the sidebar lists ZIP files in the same folder with their
sizes in MB. Click a row to open that archive; the current archive is highlighted.
Scrolling over the image turns pages in both regular-image and ZIP viewing modes.
Mouse-wheel and keyboard page turns are rate-limited to one every 100 ms.

## [Workflow] Added /.github/workflows/build.yml 
for Github Actions to build executables on workflow_dispatch (manual trigger).

Latest executables can be downloaded from (>)Actions tab of this repo.

## [Adjustments] Removed toolbar in full screen mode

## [Adjustments] Added/changed key/mouse bindings
```
[PgUp][PgDn] = Page up/down
[+] [-] = Zoom
[F5] [middle-click] = Full screen toggle
[Home] [End] = First or Last page
// [F11] was already used for windowed/maximized toggle.(by OS/DE)
```

# Work in progress:

## [] Fix frame pacing, skipping problem

## [] Jump to next folder/archive when going out of bound.	

## [SKIP] Auto maximize window on launch

## [SKIP] Auto continue from last session on launch
Use path in command line argument instead. 

Disclaimer: Almost all of the committed the codes in this fork are AI generated. 
