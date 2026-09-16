# Journal

## Features

### Piano Roll

#### Todos
- [x] Vertical + horizontal scrolling
- [x] Vim Motions
- [x] Notes are highlighted when cursor is hovering on them
- [x] Edit notes
- [ ] Change note length
- [ ] Insert mode

#### Motions
- [x] `h/l/j/k` -> Regular steps
- [x] `n/N` -> Move forward and back to next note
- [x] `w/b` -> Move forward and back to next beat
- [x] `W/B` -> Move forward and back to next bar
- [x] `e` -> Go to end of note
- [ ] `s/S` -> Move forward and back to next subdivision
- [ ] `0/$` -> Start / end of bar
- [ ] `gg/G` -> Start / end of pattern
- [ ] `{/}` -> Previous / next non-empty bar
- [ ] `H/M/L` -> Top / middle / bottom of the visible pitch range
- [ ] `zz/zt/zb` -> Centre / top / bottom the view on the cursor
- [ ] `''` -> Jump back to the previous cursor position
- [ ] `m<a-z>` / `` `<a-z> `` -> Set and jump to a mark
- [ ] `[c/]c` -> Previous / next chord (any note at any pitch, not just cursor pitch)
- [ ] Decide whether `n/N` should find *any* next note or only notes at the
      cursor pitch. Currently pitch-filtered, which means `n` finds nothing in
      a pattern unless the cursor is already on the right row.

#### Data Model
- [ ] Use the `Pattern` struct (currently dead) instead of a bare `Vec<Note>`
- [ ] Pattern length, and decide the behaviour at the end of a pattern
      (hard stop / auto-extend / loop)
- [ ] Note velocity (0-127)
- [ ] Per-pattern time signature (`beats_per_bar` exists; needs a denominator)
- [ ] Tempo (either per-pattern or project-global)
- [ ] Multiple tracks / MIDI channels visible in one roll
- [ ] Ghost notes: show another track's notes dimmed for reference
- [ ] Decide overlap policy on insert — reject, trim the existing note, or allow
      stacking. `insert_note` currently allows silent overlaps at the same pitch
- [ ] Note ties / legato joins
- [ ] Per-note micro-timing offset (nudge off the grid)
- [ ] Settle the `note_size` unit. The field is commented "in ticks" but is used
      as `note_size * PPQ`, i.e. beats — so the default of `4` is a whole bar

#### Editing
- [x] Toggle a note under the cursor (`Enter` / `Space`)
- [ ] `d` + motion -> delete notes in range (`Operator` is plumbed but unhandled)
- [ ] `dd` -> delete the note under the cursor
- [ ] `y` + motion / `p` / `P` -> yank and paste, stored as offsets relative to
      the yank origin so paste lands at the cursor
- [ ] `x` -> delete note under cursor without entering an operator
- [ ] `>/<` -> lengthen / shorten the note under the cursor by one step
- [ ] `Ctrl-a` / `Ctrl-x` -> transpose up / down by a semitone
- [ ] Transpose by an octave (`Ctrl-A` / `Ctrl-X` or a count prefix)
- [ ] Move a note in time without changing its pitch
- [ ] Duplicate a note / selection (`Shift-d` or similar)
- [ ] `r` -> replace: retune the note under the cursor to the next key pressed
- [ ] Count prefixes apply to every operator
- [ ] `.` -> repeat the last edit (already stubbed in `input.rs`)
- [ ] Quantise a selection to the current grid
- [ ] Humanise / randomise timing and velocity of a selection
- [ ] Legato: extend each note to meet the next one
- [ ] Set all selected notes to a fixed length

#### Selection (Visual Mode)
- [ ] `v` -> visual mode over a tick range
- [ ] `Ctrl-v` -> block mode over a pitch x tick rectangle
- [ ] `V` -> select whole bars
- [ ] Extend a selection with any motion
- [ ] Every operator accepts a selection as its range
- [ ] Render selected notes distinctly (theme keys `PianoRollNoteSelected` and
      `PianoRollNoteSelectedAccent` already exist but are only used for hover)
- [ ] Select all notes at the cursor pitch, or all notes in the bar
- [ ] `gv` -> reselect the previous selection

#### Insert Mode (Step Entry)
- [ ] Implement `handle_insert_mode` in `input.rs` (currently returns `None`)
- [ ] Tracker pitch layout: `z s x d c v g b h n j m` = C..B, `q 2 w 3 e r 5 t 6 y 7 u`
      an octave above
- [ ] Writing a note advances the cursor by `note_size`
- [ ] `Space` / `.` -> rest (advance without writing)
- [ ] `Backspace` -> step back and delete
- [ ] Octave up / down keys while in insert mode
- [ ] Count prefix sets the length of a single note without changing the default
- [ ] Chord entry: a "write without advancing" key, since crossterm gives no
      key-up events without the kitty keyboard protocol
- [ ] `i` / `a` -> insert at the cursor vs. after the note under the cursor
- [ ] Show the insert-mode pitch layout in a help overlay or the status line

#### Undo / Redo
- [ ] Route every mutation through a single `apply(edit)` funnel
- [ ] Undo stack with inverse operations
- [ ] Decide scope: per-window or editor-global. `EditorCommand::Undo` is already
      editor-level, so the roll needs to push onto a shared stack
- [ ] Group an insert-mode run into one undo step
- [ ] Wire up `u` (routed already) and redo (`Ctrl-r`)
- [ ] Dirty flag for unsaved changes

#### View & Rendering
- [ ] Horizontal zoom -> the `zoom` field is unused; drive `cells_per_beat` from it
- [ ] Vertical zoom (multi-row note heights)
- [ ] Configurable grid subdivision (triplets, dotted, swing)
- [ ] Snap-to-grid toggle
- [ ] Highlight the rows of the current key / scale
- [ ] Highlight octave boundaries (C rows) more strongly than other rows
- [ ] Velocity shown via note colour or a lane below the grid
- [ ] Velocity lane / automation lane under the grid
- [ ] Playhead column during playback
- [ ] Follow-playhead scrolling, toggleable
- [ ] Loop region markers
- [ ] Status line: cursor pitch, bar:beat:tick, note length, selection size
- [ ] Minimap / pattern overview
- [ ] Render the pattern-end boundary
- [ ] Only iterate visible notes when drawing, rather than the whole `Vec`
- [ ] Piano keys: drive the `pressed` highlight from real playback state
      (currently hardcoded to `vec![46, 48, 51]`)

#### Playback (depends on Audio Engine)
- [ ] Preview the note under the cursor when moving vertically ("scrub")
- [ ] Audition notes as they are entered in insert mode
- [ ] Play / pause from the cursor position
- [ ] Loop the pattern or a selected region
- [ ] Metronome
- [ ] Solo / mute a track
- [ ] Live MIDI-in recording, with a decision on whether this needs the kitty
      keyboard protocol for held-key duration

#### Ex Commands
- [ ] `:len <n>` -> set the default note length
- [ ] `:vel <n>` -> set the default velocity
- [ ] `:quantize <n>` -> quantise the selection
- [ ] `:transpose <n>` -> transpose the selection
- [ ] `:grid <n>` -> set the grid subdivision
- [ ] `:bpm <n>`, `:sig <n>/<n>` -> tempo and time signature
- [ ] `:w` / `:e` -> save and load
- [ ] Argument parsing and error reporting. `resolve_command` currently indexes
      `tokens[1]` unguarded, so `:theme` with no argument panics

#### Persistence
- [ ] Project save / load format
- [ ] MIDI file import
- [ ] MIDI file export
- [ ] Autosave / crash recovery

#### Configuration
- [ ] Keybindings loadable from config rather than hardcoded in `input.rs`
- [ ] Configurable scroll padding (currently the literal `(cells_per_bar, 5)`)
- [ ] Configurable key column width (currently the literal `7`)
- [ ] Default note length, velocity and grid as config values

#### Bugs
- [ ] `render` panics when the pane is narrower than `key_width`:
      `area.width - key_width` underflows (`pianoroll.rs:288`)
- [ ] `PianoRollMotion::Note` mutates notes from inside the *motion* handler and
      returns the cursor unchanged — it is not a motion (`:171`)
- [ ] `E` ignores its `Dir` when the cursor is already on a note, so it behaves
      identically to `e` (`:161`)
- [ ] `max_duration` never shrinks when notes are deleted, so the backwards scan
      in `note_index_at_cursor` grows monotonically slower

### Audio Engine
