# Privacy Policy

We collect no data from the user whatsoever.

## Study builds

A build running under a lab's study file keeps a local session log so the participant can hand it
to the lab. The log records:

- the wall-clock time of every line
- on app start, and again after the participant clears the log: the app version, the operating
  system name, the participant id and condition code from the study file, and the pacing values
  the file sets
- app stop, animation start and stop, pause and resume, and reminder and auto-stop firings
- the end of each phase, with its scheduled and actual duration
- each export, with the exported copy's location relative to the home folder, or only its file
  name when the copy is outside that folder

It records nothing about what else is on the screen, no keystrokes and no identity beyond the id
the lab assigned.

The log lives in the app's configuration folder. Export in Settings copies it to a folder on the
same machine: the participant's Desktop, or Documents, Downloads or the home folder when there's no
Desktop. The participant sends that copy to the lab, and exhale doesn't send it. There's no network
code in exhale. The Research item in ordinary builds asks the operating system to open a link in
the browser, and study builds leave that item out.

Clear in Settings, which appears once an export has succeeded, empties the log. Deleting the study
file turns study mode off at the next launch.
