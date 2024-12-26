# About

A minimal example for getting a terminal session working in a browser in MoonZoon.

# Running
```bash
mzoon start
```

Now open your browser and you should see a terminal session you
can type into.

# TODO

 - [ ] handle resizing
 - [ ] handle page reloads - eject terminal if new session
 - [x] re-factor frontend so that term is effectively landing page
 - [ ] support minimal re-render only on damage
 - [ ] split lib.rs into term.rs as well
 - [ ] search for TODO within the codebase
 - [x] support multiple sessions simultaneously
 - [ ] support cursor position
 - [ ] support TAB keysends
 - [ ] only send keystrokes when in focus
 - [ ] support character underlining

# Planning

 - Frontend should do a full request at least once a second
 - Backend should always send any updates as soon as they are available
