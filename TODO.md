TODO next
=========

- Word lookup
- Kanji lookup
- Pronunciation
- Wiki
  - Main functionality
  - Sentence intellisense
  - Word information
  - Cross-document table of contents
  - Ruby
    - Document-wide link style definitions `**[図書館]: としょかん`
    - Inline definitions `[図書館]^(としょかん)`
    - Always x Optional (user controlled) - `[図書館]^(!としょかん)`
    - Support for Japanese punctuation in syntax
    - Splitting `[漢字]^(kan ji)`

          <ruby>
          漢 <rp>(</rp><rt>kan</rt><rp>)</rp>
          字 <rp>(</rp><rt>ji</rt><rp>)</rp>
          </ruby>
          <style>
              rp, rt {
                  -webkit-user-select: none;
                  -moz-user-select: none;
                  -ms-user-select: none;
                  user-select: none;
              }
          </style>

- Improve sort ordering for dictionary results (match, all else...)

Major features
==============

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
- TTS system
- Word inflector (verbs, adj, ...)

Backend
=======

- Fuzzy matching in sort
- Reverse english search
- Check panic handling in requests (including logging)
- Wiki search

Client
======

- Properly organize client code
- UI persistence
- Wiki quick navigation
- Wiki editor
