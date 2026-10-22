# Privacy Policy

We collect no data from the user whatsoever.

## Study builds

A build running under a lab's study file keeps a local session log so the participant can hand it
to the lab. The log records:

- the wall-clock time of every line
- on app start, and again after the participant clears the log: the app version, the operating
  system name, the participant id and condition code from the study file, the pacing values the
  file sets, and the appearance and display values it sets (the colors, opacity, shape, gradient,
  animation mode, hold ripple and display mode)
- app stop, animation start and stop, pause and resume, each Reset, and reminder and auto-stop
  firings
- the end of each phase, with its scheduled and actual duration
- each export, with the exported copy's file name and no folder

It records nothing about what else is on the screen, no keystrokes and no identity beyond the id
the lab assigned.

The log lives in the app's configuration folder, in one file per participant id named
`exhale-log-<id>.jsonl`. Export in Settings copies it to a folder on the same machine: the first
of the participant's Desktop, Documents, Downloads and home folders that exists and takes the
copy. The participant sends that copy to the lab, and exhale doesn't send it. The operating system
can sync the Desktop to a cloud account, such as OneDrive or iCloud Drive, and that sync uploads
the copy. There's no network code in exhale. The Research item in ordinary builds asks the
operating system to open a link in the browser, and study builds leave that item out.

The log stays on disk until someone deletes it, and that includes the time after the study file is
removed. Clear in Settings empties it once an export has copied everything in it. Deleting the
study file turns study mode off at the next launch.

Every build, study or not, also keeps a diagnostic log named `exhale.log`. It sits beside the
program, or in the system's temporary folder when the program's folder isn't writable. It holds
startup steps, window and monitor changes, hotkey registration, warnings and errors, and the
details of any error in a study file. It holds no participant id and no folder path. The app
empties it at each launch, and exhale never exports or sends it.
