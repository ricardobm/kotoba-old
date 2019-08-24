import * as requestModule from 'request'
import * as util from 'util'
import * as crypto from 'crypto'
import { JSDOM } from 'jsdom'

import { Entry, EntrySource, EntryEnglish } from './dict'

const request = util.promisify(requestModule)

/**
 * Arguments to `queryJapanesePod`.
 */
export type JapanesePodArgs = {

    /** Term to lookup. */
    term: string

    /** If `true`, only look the 20,000 most common words. */
    common?: boolean

    /** If `true`, allow vulgar terms in the results. */
    vulgar?: boolean

    /** If `true`, match the start of a word instead of exactly. */
    starts?: boolean
}

/**
 * Entry from `japanesepod101.com` results.
 */
export type JapanesePodEntry = {

    /** Main japanese term. */
    term: string

    /** Kana reading of the term. */
    kana: string

    /** Audio URLs. */
    audio: string[]

    /** English definition for the term. */
    english: string

    /**
     * Additional information to append to the english definition.
     *
     * Appears italicized in grey.
     */
    english_info: string[]

    /**
     * Position of this entry in the results.
     */
    order: number
}

/**
 * Creates an `Entry` from a `JapanesePodEntry`.
 */
export function EntryFromJapanesePod(data: JapanesePodEntry): Entry {
    const entry: Entry = {
        source:         EntrySource.JapanesePod,
        origin:         '',
        expression:     data.term,
        reading:        data.kana,
        score:         -data.order,
        extra_forms:    [],
        extra_readings: [],
        tags:           [],
        english: [
            {
                glossary: [data.english],
                tags:     [],
                info:     data.english_info,
                links:    [],
            },
        ],
    }
    return entry
}

/**
 * Query `japanesepod101.com` dictionary.
 */
export async function queryJapanesePod(args: JapanesePodArgs) {
    const params: {[key:string]: string} = {
        post: 'dictionary_reference',
        match_type: args.starts ? 'starts' : 'exact',
        search_query: args.term,
    }
    if (args.vulgar) {
        params.vulgar = 'true'
    }
    if (args.common) {
        params.common = 'true'
    }

    const response = await request({
        method: 'POST',
        uri:    'https://www.japanesepod101.com/learningcenter/reference/dictionary_post',
        form:   params,
    })

    const results: JapanesePodEntry[] = []

    const dom = new JSDOM(response.body)
    const doc = dom.window.document
    const rows = doc.querySelectorAll('div.dc-result-row')
    rows.forEach((row, order) => {
        const termEl = row.querySelector('span.dc-vocab')
        const term = (termEl && termEl.textContent || '').trim()

        const kanaEl = row.querySelector('span.dc-vocab_kana')
        const kana = (kanaEl && kanaEl.textContent || '').trim()

        const audio: string[] = []
        row.querySelectorAll('div.di-player:first-of-type audio > source').forEach(el => {
            const src = el.getAttribute('src')
            src && audio.push(src)
        })

        const englishEl = row.querySelector('span.dc-english')
        const english_info: string[] = []
        if (englishEl) {
            const toRemove: Element[] = []
            englishEl.querySelectorAll('span.dc-english-grey').forEach(el => {
                const info = (el.textContent || '').trim()
                english_info.push(info)
                toRemove.push(el)
            })
            for (const el of toRemove) {
                el.remove()
            }
        }

        const english = (englishEl && englishEl.textContent || '').trim()

        if (term || kana) {
            results.push({ term, kana, audio, english, english_info, order })
        }
    })

    return results
}

/**
 * Load an audio file from `assets.languagepod101.com` given the word and its
 * kana reading.
 */
export async function loadJapanesePodAudio(kanji: string, kana: string) {

    const BLACKLIST_HASH   = 'ae6398b5a27bc8c0a771df6c907ade794be15518174773c58c7c7ddd17098906'

    const response = await request({
        method:   'GET',
        uri:      'https://assets.languagepod101.com/dictionary/japanese/audiomp3.php',
        qs:       { kanji, kana },
        encoding: null,
    })

    const data = response.body as Buffer
    if (!data) {
        return null
    }

    const hash = crypto.createHash('sha256').update(data).digest('hex')
    if (hash === BLACKLIST_HASH) {
        return null
    }

    return { hash, data, name: `${kanji}_${kana}.mp3` }
}
