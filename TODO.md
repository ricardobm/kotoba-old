Major features
==============

- Proper pronunciation API
  - Load from Forvo only as a last resort (too slow)
    - Maybe have a first pass that loads only the main entries and a second
      pass for only the other detailed entries.
  - Properly time loading
- De-inflect words
- Wiki
- Favorite words and personal database
- Media file support
- Subtitles & Karaoke mode
- Crossword game
- Typing game
- Kanji view & drawing
- Kana table & drawing
- Anki integration
- Spaced repetition system
- Flash cards
- Sentences
- TTS system
- Word inflector (verbs, adj, ...)

Backend
=======

- Wrapper type for dictionary returns
- Return all kanji from query results
- Improve sort ordering for results (match, all else...)
- Fuzzy matching in sort
- Reverse english search

Client
======

- Properly organize client code
- Use redux
- SASS
- Find a UI library
- UI persistence
- History navigation
- Debounce calls
